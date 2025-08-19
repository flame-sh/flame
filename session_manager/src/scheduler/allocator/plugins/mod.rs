/*
Copyright 2025 The Flame Authors.
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
use std::sync::Arc;

use stdng::collections;

use crate::model::{ExecutorInfoPtr, NodeInfo, NodeInfoPtr, SessionInfo, SessionInfoPtr, SnapShot};
use crate::scheduler::allocator::plugins::fairshare::FairShare;
use crate::scheduler::Context;

use common::ptr::{self, MutexPtr};
use common::{lock_ptr, FlameError};

mod fairshare;

// lazy_static! {
//     static ref INSTANCE: MutexPtr<PluginManager> = Arc::new(Mutex::new(PluginManager {
//         plugins: HashMap::from([("fairshare".to_string(), FairShare::new_ptr())])
//     }));
// }

pub type PluginPtr = Box<dyn Plugin>;
pub type PluginManagerPtr = Arc<PluginManager>;

pub trait Plugin: Send + Sync + 'static {
    // Installation of plugin
    fn setup(&mut self, ss: &SnapShot) -> Result<(), FlameError>;

    // Schedule Fn
    fn ssn_order_fn(&self, s1: &SessionInfo, s2: &SessionInfo) -> Option<Ordering>;

    fn node_order_fn(&self, s1: &NodeInfo, s2: &NodeInfo) -> Option<Ordering>;

    fn is_underused(&self, ssn: &SessionInfoPtr) -> Option<bool>;

    fn is_allocatable(&self, node: &NodeInfoPtr, ssn: &SessionInfoPtr) -> Option<bool>;
    fn is_reclaimable(&self, exec: &ExecutorInfoPtr) -> Option<bool>;

    // Events
    fn on_create_executor(&mut self, node: NodeInfoPtr, ssn: SessionInfoPtr);
}

pub struct PluginManager {
    pub plugins: MutexPtr<HashMap<String, PluginPtr>>,
}

impl PluginManager {
    pub fn setup(ss: &SnapShot) -> Result<PluginManagerPtr, FlameError> {
        let mut plugins = HashMap::from([("fairshare".to_string(), FairShare::new_ptr())]);

        for plugin in plugins.values_mut() {
            plugin.setup(ss)?;
        }

        Ok(Arc::new(PluginManager {
            plugins: ptr::new_ptr(plugins),
        }))
    }

    pub fn is_underused(&self, ssn: &SessionInfoPtr) -> Result<bool, FlameError> {
        let plugins = lock_ptr!(self.plugins)?;

        Ok(plugins
            .values()
            .all(|plugin| plugin.is_underused(ssn).unwrap_or(false)))
    }

    pub fn is_allocatable(
        &self,
        node: &NodeInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<bool, FlameError> {
        let plugins = lock_ptr!(self.plugins)?;

        Ok(plugins
            .values()
            .all(|plugin| plugin.is_allocatable(node, ssn).unwrap_or(false)))
    }

    pub fn is_reclaimable(&self, exec: &ExecutorInfoPtr) -> Result<bool, FlameError> {
        let plugins = lock_ptr!(self.plugins)?;

        Ok(plugins
            .values()
            .all(|plugin| plugin.is_reclaimable(exec).unwrap_or(false)))
    }

    pub fn on_create_executor(
        &self,
        node: NodeInfoPtr,
        ssn: SessionInfoPtr,
    ) -> Result<(), FlameError> {
        let mut plugins = lock_ptr!(self.plugins)?;

        for plugin in plugins.values_mut() {
            plugin.on_create_executor(node.clone(), ssn.clone());
        }

        Ok(())
    }

    pub fn ssn_order_fn(&self, t1: &SessionInfoPtr, t2: &SessionInfoPtr) -> Ordering {
        if let Ok(plugins) = lock_ptr!(self.plugins) {
            for plugin in plugins.values() {
                if let Some(order) = plugin.ssn_order_fn(t1, t2) {
                    if order != Ordering::Equal {
                        return order;
                    }
                }
            }
        }

        Ordering::Equal
    }

    pub fn node_order_fn(&self, t1: &NodeInfoPtr, t2: &NodeInfoPtr) -> Ordering {
        if let Ok(plugins) = lock_ptr!(self.plugins) {
            for plugin in plugins.values() {
                if let Some(order) = plugin.node_order_fn(t1, t2) {
                    if order != Ordering::Equal {
                        return order;
                    }
                }
            }
        }
        Ordering::Equal
    }
}
