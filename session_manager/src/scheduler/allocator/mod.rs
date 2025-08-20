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

mod plugins;

use std::cmp::Ordering;
use std::sync::Arc;
use stdng::collections;

use crate::controller::ControllerPtr;
use crate::model::{NodeInfo, NodeInfoPtr, SessionInfoPtr, SnapShotPtr};
use crate::scheduler::allocator::plugins::PluginManager;
use crate::scheduler::allocator::plugins::PluginManagerPtr;
use crate::scheduler::Context;
use common::apis::{Node, Session};
use common::FlameError;

pub struct Allocator {
    pub snapshot: SnapShotPtr,
    pub controller: ControllerPtr,
    pub plugins: PluginManagerPtr,
}

pub type AllocatorPtr = Arc<Allocator>;

impl Allocator {
    pub fn new(snapshot: SnapShotPtr, controller: ControllerPtr) -> Result<Self, FlameError> {
        Ok(Self {
            snapshot: snapshot.clone(),
            controller,
            plugins: PluginManager::setup(&snapshot)?,
        })
    }

    pub fn is_underused(&self, ssn: &SessionInfoPtr) -> Result<bool, FlameError> {
        self.plugins.is_underused(ssn)
    }

    pub fn is_allocatable(
        &self,
        node: &NodeInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<bool, FlameError> {
        self.plugins.is_allocatable(node, ssn)
    }

    pub async fn create_executor(
        &self,
        node: NodeInfoPtr,
        ssn: SessionInfoPtr,
    ) -> Result<(), FlameError> {
        self.plugins.on_create_executor(node.clone(), ssn.clone())?;

        // Create executor in controller
        let exec = self
            .controller
            .create_executor(node.name.clone(), ssn.id)
            .await?;
        log::debug!(
            "Created executor <{}> for session <{}> on node <{}>",
            exec.id,
            ssn.id,
            node.name
        );

        Ok(())
    }
}

pub fn ssn_order_fn(ctx: &Context) -> impl collections::Cmp<SessionInfoPtr> {
    SsnOrderFn {
        plugin_mgr: ctx.allocator.plugins.clone(),
    }
}

struct SsnOrderFn {
    plugin_mgr: PluginManagerPtr,
}

impl collections::Cmp<SessionInfoPtr> for SsnOrderFn {
    fn cmp(&self, t1: &SessionInfoPtr, t2: &SessionInfoPtr) -> Ordering {
        self.plugin_mgr.ssn_order_fn(t1, t2)
    }
}

pub fn node_order_fn(ctx: &Context) -> impl collections::Cmp<NodeInfoPtr> {
    NodeOrderFn {
        plugin_mgr: ctx.allocator.plugins.clone(),
    }
}

struct NodeOrderFn {
    plugin_mgr: PluginManagerPtr,
}

impl collections::Cmp<NodeInfoPtr> for NodeOrderFn {
    fn cmp(&self, t1: &NodeInfoPtr, t2: &NodeInfoPtr) -> Ordering {
        self.plugin_mgr.node_order_fn(t1, t2)
    }
}
