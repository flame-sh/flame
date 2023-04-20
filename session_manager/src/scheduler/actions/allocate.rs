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

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use std::sync::Arc;

use common::apis::{ExecutorState, SessionID, SessionState, TaskState};
use crate::scheduler::actions::Action;
use crate::storage::Storage;
use crate::FlameError;
use crate::model::SnapShot;

pub struct AllocateAction {
    pub storage: Arc<Storage>,
}

struct SsnOrderInfo {
    id: SessionID,
    slots: i32,
    desired: f64,
    allocated: f64,
}

impl Eq for SsnOrderInfo {}

impl PartialEq<Self> for SsnOrderInfo {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd<Self> for SsnOrderInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SsnOrderInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        let my_diff = self.desired - self.allocated;
        let other_diff = other.desired - other.allocated;

        let res = Ordering::Equal;

        if other_diff > my_diff {
            return Ordering::Greater;
        }

        if other_diff < my_diff {
            return Ordering::Less;
        }

        res
    }
}

impl Action for AllocateAction {
    fn execute(&self, ss: &mut SnapShot) -> Result<(), FlameError> {
        log::debug!(
            "Session: <{}>, Executor: <{}>",
            ss.sessions.len(),
            ss.executors.len()
        );

        let mut ssn_order_info = BinaryHeap::new();

        // TODO(k82cn): move this into SsnOrderFn plugin.
        if let Some(ssn_list) = ss.ssn_state_index.get(&SessionState::Open) {
            for ssn in ssn_list {
                let mut desired = 0.0;
                for p in vec![TaskState::Pending, TaskState::Running] {
                    if let Some(s) = ssn.tasks_status.get(&p) {
                        desired += *s as f64 * (ssn.slots as f64);
                    }
                }

                let allocated = ssn.executors.len() as f64 * (ssn.slots as f64);

                log::debug!(
                    "Session <{}>: desired <{}>, allocated<{}>",
                    ssn.id.clone(),
                    desired.clone(),
                    allocated.clone()
                );

                ssn_order_info.push(SsnOrderInfo {
                    id: ssn.id.clone(),
                    slots: ssn.slots,
                    desired,
                    allocated,
                })
            }
        }

        loop {
            if ssn_order_info.is_empty() {
                break;
            }

            if let Some(mut ssn) = ssn_order_info.pop() {
                if ssn.allocated > ssn.desired {
                    continue;
                }

                if let Some(idle_execs) = ss.exec_state_index.get_mut(&ExecutorState::Idle) {
                    let mut pos = -1;
                    for (i, exec) in idle_execs.iter().enumerate() {
                        // TODO(k82cn): filter Executor by slots & applications.
                        if let Err(e) = self.storage.bind_session(exec.id.clone(), ssn.id.clone()) {
                            log::error!(
                                "Failed to bind Session <{}> to Executor <{}>: {}.",
                                exec.id.clone(),
                                ssn.id.clone(),
                                e
                            );
                            continue;
                        }

                        pos = i as i32;
                        ssn.allocated += ssn.slots as f64;
                        ssn_order_info.push(ssn);
                        break;
                    }

                    // TODO(k82cn): also remove it from ss.executors & ss.exec_index to keep data sync.
                    if pos >= 0 {
                        idle_execs.remove(pos as usize);
                    }
                }
            }
        }

        Ok(())
    }
}
