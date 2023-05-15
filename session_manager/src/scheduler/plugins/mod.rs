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
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use common::{lock_ptr, FlameError};
use lazy_static::lazy_static;

use crate::model::{ExecutorInfoPtr, SessionInfo, SessionInfoPtr, SnapShot};
use crate::scheduler::plugins::fairshare::FairShare;
use common::ptr::MutexPtr;

mod fairshare;

lazy_static! {
    static ref INSTANCE: MutexPtr<PluginManager> = Arc::new(Mutex::new(PluginManager {
        plugins: HashMap::from([("fairshare".to_string(), FairShare::new_ptr())])
    }));
}

pub type PluginPtr = Box<dyn Plugin>;
pub type PluginManagerPtr = MutexPtr<PluginManager>;

pub trait Plugin: Send + Sync + 'static {
    // Installation of plugin
    fn setup(&mut self, ss: &SnapShot);

    // Schedule Fn
    fn ssn_order_fn(&self, s1: &SessionInfo, s2: &SessionInfo) -> Option<Ordering>;

    fn is_underused(&self, ssn: &SessionInfoPtr) -> Option<bool>;

    fn is_preemptible(&self, ssn: &SessionInfoPtr) -> Option<bool>;

    fn filter(
        &self,
        exec: &[ExecutorInfoPtr],
        ssn: &SessionInfoPtr,
    ) -> Option<Vec<ExecutorInfoPtr>>;

    // Events
    fn on_session_bind(&mut self, ssn: &SessionInfoPtr);
    fn on_session_unbind(&mut self, ssn: &SessionInfoPtr);
}

pub struct PluginManager {
    pub plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn setup(ss: &SnapShot) -> Result<PluginManagerPtr, FlameError> {
        let mut ins = lock_ptr!(INSTANCE)?;
        for plugin in ins.plugins.values_mut() {
            plugin.setup(ss);
        }

        Ok(INSTANCE.clone())
    }

    pub fn is_underused(&self, ssn: &SessionInfoPtr) -> bool {
        self.plugins
            .values()
            .all(|plugin| plugin.is_underused(ssn).unwrap_or(false))
    }

    pub fn is_preemptible(&self, ssn: &SessionInfoPtr) -> bool {
        self.plugins
            .values()
            .all(|plugin| plugin.is_preemptible(ssn).unwrap_or(false))
    }

    pub fn filter(
        &self,
        execs: &Vec<ExecutorInfoPtr>,
        ssn: &SessionInfoPtr,
    ) -> Vec<ExecutorInfoPtr> {
        let mut res = vec![];
        for exec in execs {
            if exec
                .applications
                .iter()
                .any(|app| app.name == ssn.application)
            {
                res.push(exec.clone());
            }
        }

        // TODO(k82cn): also filter Executor by Plugins.

        res
    }

    pub fn on_session_bind(&mut self, ssn: &SessionInfoPtr) {
        for plugin in self.plugins.values_mut() {
            plugin.on_session_bind(ssn)
        }
    }

    pub fn on_session_unbind(&mut self, ssn: &SessionInfoPtr) {
        for plugin in self.plugins.values_mut() {
            plugin.on_session_unbind(ssn)
        }
    }
}

impl PartialOrd<Self> for SessionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<Self> for SessionInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SessionInfo {}

impl Ord for SessionInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut ins = lock_ptr!(INSTANCE);
        match &mut ins {
            Ok(ins) => {
                for plugin in ins.plugins.values() {
                    if let Some(order) = plugin.ssn_order_fn(self, other) {
                        if order != Ordering::Equal {
                            return order;
                        }
                    }
                }

                Ordering::Equal
            }
            Err(e) => {
                log::warn!("Failed to compare sessions: {}", e);
                Ordering::Equal
            }
        }
    }
}
