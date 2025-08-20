/*
Copyright 2023 The Flame Authors.
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

use crate::model::{BOUND_EXECUTOR, OPEN_SESSION};
use crate::scheduler::actions::{Action, ActionPtr};
use crate::scheduler::ctx::Context;
use crate::scheduler::dispatcher::ssn_order_fn;

use common::FlameError;
use common::{trace::TraceFn, trace_fn};

pub struct ShuffleAction {}

impl ShuffleAction {
    pub fn new_ptr() -> ActionPtr {
        Arc::new(ShuffleAction {})
    }
}

#[async_trait::async_trait]
impl Action for ShuffleAction {
    async fn execute(&self, ctx: &mut Context) -> Result<(), FlameError> {
        trace_fn!("ShuffleAction::execute");
        let ss = ctx.snapshot.clone();

        let mut underused = BinaryHeap::new(ssn_order_fn(ctx));
        let open_ssns = ss.find_sessions(OPEN_SESSION)?;
        for ssn in open_ssns.values() {
            if ctx.dispatcher.is_underused(ssn)? {
                underused.push(ssn.clone());
            }
        }

        let mut bound_execs = vec![];
        let execs = ss.find_executors(BOUND_EXECUTOR)?;
        for exec in execs.values() {
            bound_execs.push(exec.clone());
        }

        loop {
            if underused.is_empty() {
                break;
            }

            let ssn = underused.pop().unwrap();
            if !ctx.dispatcher.is_underused(&ssn)? {
                continue;
            }

            let mut pos = None;
            for (i, exec) in bound_execs.iter().enumerate() {
                if !ctx.dispatcher.filter_one(exec, &ssn) {
                    continue;
                }

                let target_ssn = match exec.ssn_id {
                    Some(ssn_id) => Some(ss.get_session(&ssn_id)?),
                    None => None,
                };

                if let Some(target_ssn) = target_ssn {
                    if !ctx.dispatcher.is_preemptible(&target_ssn)? {
                        continue;
                    }

                    ctx.dispatcher
                        .unbind_session(exec.clone(), target_ssn.clone())
                        .await?;
                    ctx.dispatcher
                        .pipeline_session(exec.clone(), ssn.clone())
                        .await?;
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
