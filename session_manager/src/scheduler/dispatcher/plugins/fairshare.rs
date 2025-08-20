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

use std::cmp::Ordering;
use std::collections::binary_heap::BinaryHeap;
use std::collections::HashMap;

use crate::model::{
    ExecutorInfoPtr, SessionInfo, SessionInfoPtr, SnapShot, ALL_APPLICATION, ALL_EXECUTOR,
    OPEN_SESSION,
};
use crate::scheduler::dispatcher::plugins::{Plugin, PluginPtr};
use common::apis::{SessionID, TaskState};
use common::FlameError;

#[derive(Default, Clone)]
struct SSNInfo {
    pub id: SessionID,
    pub slots: i32,
    pub desired: f64,
    pub deserved: f64,
    pub allocated: f64,
}

impl Eq for SSNInfo {}

impl PartialEq<Self> for SSNInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd<Self> for SSNInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SSNInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.deserved < other.deserved {
            return Ordering::Greater;
        }

        if self.deserved > other.deserved {
            return Ordering::Less;
        }

        Ordering::Equal
    }
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
    fn setup(&mut self, ss: &SnapShot) -> Result<(), FlameError> {
        let open_ssns = ss.find_sessions(OPEN_SESSION)?;

        let apps = ss.find_applications(ALL_APPLICATION)?;

        log::debug!(
            "There are {} open sessions, {} applications.",
            open_ssns.len(),
            apps.len()
        );

        for ssn in open_ssns.values() {
            let mut desired = 0.0;
            for state in [TaskState::Pending, TaskState::Running] {
                if let Some(d) = ssn.tasks_status.get(&state) {
                    desired += *d as f64 * ssn.slots as f64;
                }
            }

            if let Some(app) = apps.get(&ssn.application) {
                desired = desired.min((app.max_instances * ssn.slots) as f64);

                self.ssn_map.insert(
                    ssn.id,
                    SSNInfo {
                        id: ssn.id,
                        desired,
                        slots: ssn.slots,
                        ..SSNInfo::default()
                    },
                );
            } else {
                log::warn!(
                    "Application <{}> not found for session <{}>.",
                    ssn.application,
                    ssn.id
                );
            }
        }

        let mut remaining_slots = 0.0;

        let executors = ss.find_executors(ALL_EXECUTOR)?;
        for exe in executors.values() {
            remaining_slots += exe.resreq.to_slots(&ss.unit) as f64;
            if let Some(ssn_id) = exe.ssn_id {
                if let Some(ssn) = self.ssn_map.get_mut(&ssn_id) {
                    ssn.allocated += ssn.slots as f64;
                }
            }
        }

        let mut underused = BinaryHeap::from_iter(self.ssn_map.values_mut());
        loop {
            if remaining_slots < 0.001 {
                break;
            }

            if underused.is_empty() {
                break;
            }

            let delta = remaining_slots / underused.len() as f64;
            let ssn = underused.pop().unwrap();

            if ssn.deserved + delta < ssn.desired {
                ssn.deserved += delta;
                remaining_slots -= delta;
                underused.push(ssn);
            } else {
                remaining_slots -= ssn.desired - ssn.deserved;
                ssn.deserved = ssn.desired;
            }
        }

        if log::log_enabled!(log::Level::Debug) {
            for ssn in self.ssn_map.values() {
                log::debug!(
                    "Session <{}>: slots <{}>, desired <{}>, deserved <{}>, allocated <{}>.",
                    ssn.id,
                    ssn.slots,
                    ssn.desired,
                    ssn.deserved,
                    ssn.allocated
                )
            }
        }

        Ok(())
    }

    fn ssn_order_fn(&self, s1: &SessionInfo, s2: &SessionInfo) -> Option<Ordering> {
        let ss1 = self.ssn_map.get(&s1.id);
        let ss2 = self.ssn_map.get(&s2.id);

        if ss1.is_none() || ss2.is_none() {
            return None;
        }

        let ss1 = ss1.unwrap();
        let ss2 = ss2.unwrap();

        let left = ss1.allocated * ss2.deserved;
        let right = ss2.allocated * ss1.deserved;

        if left < right {
            return Some(Ordering::Greater);
        }

        if left > right {
            return Some(Ordering::Less);
        }

        Some(Ordering::Equal)
    }

    fn is_underused(&self, ssn: &SessionInfoPtr) -> Option<bool> {
        self.ssn_map
            .get(&ssn.id)
            .map(|ssn| ssn.allocated < ssn.deserved)
    }

    fn is_preemptible(&self, ssn: &SessionInfoPtr) -> Option<bool> {
        self.ssn_map
            .get(&ssn.id)
            .map(|ssn| ssn.allocated - ssn.slots as f64 >= ssn.deserved)
    }

    fn filter(
        &self,
        _exec: &[ExecutorInfoPtr],
        _ssn: &SessionInfoPtr,
    ) -> Option<Vec<ExecutorInfoPtr>> {
        None
    }

    fn on_session_bind(&mut self, ssn: SessionInfoPtr) {
        if let Some(ss) = self.ssn_map.get_mut(&ssn.id) {
            ss.allocated += ssn.slots as f64;
        }
    }

    fn on_session_unbind(&mut self, ssn: SessionInfoPtr) {
        if let Some(ss) = self.ssn_map.get_mut(&ssn.id) {
            ss.allocated -= ssn.slots as f64;
        }
    }
}
