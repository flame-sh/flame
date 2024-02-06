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

use std::collections::HashMap;
use stdng::collections::BinaryHeap;

use std::sync::Arc;

use crate::scheduler::actions::{Action, ActionPtr};
use crate::scheduler::plugins::ssn_order_fn;
use crate::scheduler::Context;

use crate::FlameError;
use common::apis::{ExecutorState, SessionState};
use common::{trace::TraceFn, trace_fn};

pub struct AllocateAction {}

impl AllocateAction {
    pub fn new_ptr() -> ActionPtr {
        Arc::new(AllocateAction {})
    }
}

impl Action for AllocateAction {
    fn execute(&self, ctx: &mut Context) -> Result<(), FlameError> {
        trace_fn!("AllocateAction::execute");
        let ss = ctx.snapshot.borrow().clone();

        log::debug!(
            "Session: <{}>, Executor: <{}>",
            ss.ssn_index
                .get(&SessionState::Open)
                .unwrap_or(&HashMap::new())
                .len(),
            ss.executors.len()
        );

        let mut open_ssns = BinaryHeap::new(ssn_order_fn(ctx));
        let mut idle_execs = Vec::new();

        if let Some(ssn_list) = ss.ssn_index.get(&SessionState::Open) {
            for ssn in ssn_list.values() {
                open_ssns.push(ssn.clone());
            }
        }

        if let Some(execs) = ss.exec_index.get(&ExecutorState::Idle) {
            for exec in execs.values() {
                idle_execs.push(exec.clone());
            }
        }

        loop {
            if open_ssns.is_empty() {
                break;
            }

            let ssn = open_ssns.pop().unwrap();
            log::debug!("Start resources allocation for session <{}>", &ssn.id);
            if !ctx.is_underused(&ssn) {
                continue;
            }

            log::debug!(
                "Session <{}> is underused, start to allocate resources.",
                &ssn.id
            );

            let mut pos = None;
            for (i, exec) in idle_execs.iter_mut().enumerate() {
                log::debug!(
                    "Try to allocate executor <{}> for session <{}>",
                    exec.id.clone(),
                    ssn.id.clone()
                );

                if !ctx.filter_one(exec, &ssn) {
                    continue;
                }

                if let Err(e) = ctx.bind_session(exec, &ssn) {
                    log::error!(
                        "Failed to bind Session <{}> to Executor <{}>: {}.",
                        exec.id.clone(),
                        ssn.id.clone(),
                        e
                    );
                    continue;
                }

                pos = Some(i);

                log::debug!(
                    "Executor <{}> was allocated to session <{}>, remove it from idle list.",
                    exec.id.clone(),
                    ssn.id.clone()
                );
                open_ssns.push(ssn);
                break;
            }

            if let Some(p) = pos {
                idle_execs.remove(p);
            }
        }

        Ok(())
    }
}
