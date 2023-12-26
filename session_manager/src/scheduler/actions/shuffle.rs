/*
Copyright 2023 The xflops Authors.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::sync::Arc;

use stdng::collections::BinaryHeap;

use crate::scheduler::actions::{Action, ActionPtr};
use crate::scheduler::ctx::Context;
use crate::scheduler::plugins::ssn_order_fn;

use common::apis::{ExecutorState, SessionState};
use common::FlameError;
use common::{trace::TraceFn, trace_fn};

pub struct ShuffleAction {}

impl ShuffleAction {
    pub fn new_ptr() -> ActionPtr {
        Arc::new(ShuffleAction {})
    }
}

impl Action for ShuffleAction {
    fn execute(&self, ctx: &mut Context) -> Result<(), FlameError> {
        trace_fn!("ShuffleAction::execute");
        let ss = ctx.snapshot.borrow().clone();

        let mut underused = BinaryHeap::new(ssn_order_fn(ctx));
        if let Some(open_ssns) = ss.ssn_index.get(&SessionState::Open) {
            for ssn in open_ssns.values() {
                if ctx.is_underused(ssn) {
                    underused.push(ssn.clone());
                }
            }
        }

        let mut bound_execs = vec![];
        if let Some(execs) = ss.exec_index.get(&ExecutorState::Bound) {
            for exec in execs.values() {
                bound_execs.push(exec.clone());
            }
        }

        loop {
            if underused.is_empty() {
                break;
            }

            let ssn = underused.pop().unwrap();
            if !ctx.is_underused(&ssn) {
                continue;
            }

            let mut pos = None;
            for (i, exec) in bound_execs.iter().enumerate() {
                if !ctx.filter_one(exec, &ssn) {
                    continue;
                }

                let target_ssn = match exec.ssn_id {
                    Some(ssn_id) => ss.sessions.get(&ssn_id).cloned(),
                    None => None,
                };

                if let Some(target_ssn) = target_ssn {
                    if !ctx.is_preemptible(&target_ssn) {
                        continue;
                    }

                    if let Err(e) = ctx.unbind_session(exec, &target_ssn) {
                        log::error!(
                            "Failed to unbind Session <{}> to Executor <{}>: {}.",
                            exec.id.clone(),
                            target_ssn.id.clone(),
                            e
                        );
                        continue;
                    }
                    if let Err(e) = ctx.pipeline_session(exec, &ssn) {
                        log::error!(
                            "Failed to pipeline Session <{}> to Executor <{}>: {}.",
                            exec.id.clone(),
                            ssn.id.clone(),
                            e
                        );
                        continue;
                    }
                }

                pos = Some(i);
                log::debug!(
                    "Executor <{}> was pipeline to session <{}>, remove it from bound list.",
                    exec.id.clone(),
                    ssn.id.clone()
                );
                underused.push(ssn.clone());
                break;
            }

            if let Some(p) = pos {
                bound_execs.remove(p);
            }
        }

        Ok(())
    }
}
