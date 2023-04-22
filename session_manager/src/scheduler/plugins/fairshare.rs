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

use crate::model::{ExecutorInfoPtr, SessionInfo, SessionInfoPtr, SnapShot};
use crate::scheduler::plugins::{Plugin, PluginPtr};
use common::apis::{SessionID, SessionState, TaskState};
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Default)]
struct SSNInfo {
    pub desired: f64,
    pub allocated: f64,
}

pub struct FairShare {
    ssn_map: HashMap<SessionID, SSNInfo>,
}

impl FairShare {
    pub fn new_ptr() -> PluginPtr {
        Box::new(FairShare {
            ssn_map: HashMap::new(),
        })
    }
}

impl Plugin for FairShare {
    fn setup(&mut self, ss: &SnapShot) {
        let empty_map = HashMap::new();
        let open_ssns = ss.ssn_index.get(&SessionState::Open).unwrap_or(&empty_map);

        for ssn in open_ssns.values() {
            let mut desired = 0.0;
            for state in [TaskState::Pending, TaskState::Running] {
                if let Some(d) = ssn.tasks_status.get(&state) {
                    desired += *d as f64 * ssn.slots as f64;
                }
            }

            self.ssn_map.insert(
                ssn.id,
                SSNInfo {
                    desired,
                    ..SSNInfo::default()
                },
            );
        }

        for exe in ss.executors.values() {
            if let Some(ssn_id) = exe.ssn_id {
                if let Some(ssn) = self.ssn_map.get_mut(&ssn_id) {
                    ssn.allocated += exe.slots as f64;
                }
            }
        }
    }

    fn ssn_order_fn(&self, s1: &SessionInfo, s2: &SessionInfo) -> Option<Ordering> {
        let ss1 = self.ssn_map.get(&s1.id);
        let ss2 = self.ssn_map.get(&s2.id);

        if ss1.is_none() || ss2.is_none() {
            return None;
        }

        let ss1 = ss1.unwrap();
        let ss2 = ss2.unwrap();

        let left = ss1.allocated * ss2.desired;
        let right = ss2.allocated * ss1.desired;

        if left > right {
            return Some(Ordering::Greater);
        }

        if left < right {
            return Some(Ordering::Less);
        }

        Some(Ordering::Equal)
    }

    fn is_underused(&self, ssn: &SessionInfoPtr) -> Option<bool> {
        self.ssn_map
            .get(&ssn.id)
            .map(|ssn| ssn.allocated < ssn.desired)
    }

    fn filter(
        &self,
        _exec: &[ExecutorInfoPtr],
        _ssn: &SessionInfoPtr,
    ) -> Option<Vec<ExecutorInfoPtr>> {
        None
    }

    fn on_session_bind(&mut self, ssn: &SessionInfoPtr) {
        if let Some(ss) = self.ssn_map.get_mut(&ssn.id) {
            ss.allocated += ssn.slots as f64;
        }
    }

    fn on_session_unbind(&mut self, ssn: &SessionInfoPtr) {
        if let Some(ss) = self.ssn_map.get_mut(&ssn.id) {
            ss.allocated -= ssn.slots as f64;
        }
    }
}
