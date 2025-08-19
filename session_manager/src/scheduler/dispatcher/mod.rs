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
use crate::model::{ExecutorInfoPtr, SessionInfoPtr, SnapShotPtr};
use crate::scheduler::dispatcher::plugins::PluginManager;
use crate::scheduler::dispatcher::plugins::PluginManagerPtr;
use crate::scheduler::Context;

use common::apis::ExecutorState;
use common::FlameError;

pub type DispatcherPtr = Arc<Dispatcher>;

pub struct Dispatcher {
    pub snapshot: SnapShotPtr,
    pub controller: ControllerPtr,
    pub plugins: PluginManagerPtr,
}

impl Dispatcher {
    pub fn new(snapshot: SnapShotPtr, controller: ControllerPtr) -> Result<Self, FlameError> {
        Ok(Self {
            snapshot: snapshot.clone(),
            controller,
            plugins: PluginManager::setup(&snapshot)?,
        })
    }

    pub fn filter(&self, execs: &[ExecutorInfoPtr], ssn: &SessionInfoPtr) -> Vec<ExecutorInfoPtr> {
        self.plugins.filter(execs, ssn)
    }

    pub fn filter_one(&self, exec: &ExecutorInfoPtr, ssn: &SessionInfoPtr) -> bool {
        !self.filter(&[exec.clone()], ssn).is_empty()
    }

    pub fn is_underused(&self, ssn: &SessionInfoPtr) -> Result<bool, FlameError> {
        self.plugins.is_underused(ssn)
    }

    pub fn is_preemptible(&self, ssn: &SessionInfoPtr) -> Result<bool, FlameError> {
        self.plugins.is_preemptible(ssn)
    }

    pub async fn bind_session(
        &self,
        exec: ExecutorInfoPtr,
        ssn: SessionInfoPtr,
    ) -> Result<(), FlameError> {
        self.controller
            .bind_session(exec.id.clone(), ssn.id)
            .await?;
        self.plugins.on_session_bind(ssn)?;
        self.snapshot
            .update_executor_state(exec.clone(), ExecutorState::Binding)?;

        Ok(())
    }

    pub async fn pipeline_session(
        &self,
        exec: ExecutorInfoPtr,
        ssn: SessionInfoPtr,
    ) -> Result<(), FlameError> {
        self.plugins.on_session_bind(ssn)?;

        self.snapshot
            .update_executor_state(exec.clone(), ExecutorState::Binding)?;

        Ok(())
    }

    pub async fn unbind_session(
        &self,
        exec: ExecutorInfoPtr,
        ssn: SessionInfoPtr,
    ) -> Result<(), FlameError> {
        self.controller.unbind_executor(exec.id.clone()).await?;
        self.plugins.on_session_unbind(ssn)?;
        self.snapshot
            .update_executor_state(exec.clone(), ExecutorState::Unbinding)?;

        Ok(())
    }
}

pub fn ssn_order_fn(ctx: &Context) -> impl collections::Cmp<SessionInfoPtr> {
    SsnOrderFn {
        plugin_mgr: ctx.dispatcher.plugins.clone(),
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
