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

use crate::model::{ExecutorInfoPtr, SessionInfoPtr, SnapShot};
use crate::scheduler::actions::{ActionPtr, AllocateAction, BackfillAction, ShuffleAction};
use crate::scheduler::plugins::{PluginManager, PluginManagerPtr};
use crate::storage;
use crate::storage::StoragePtr;

use common::apis::ExecutorState;
use common::ctx::FlameContext;
use common::{lock_ptr, FlameError};

const DEFAULT_SCHEDULE_INTERVAL: u64 = 500;

pub struct Context {
    pub snapshot: SnapShot,
    pub storage: StoragePtr,
    pub actions: Vec<ActionPtr>,
    pub plugins: PluginManagerPtr,
    pub schedule_interval: u64,
}

impl TryFrom<&FlameContext> for Context {
    type Error = FlameError;

    fn try_from(_: &FlameContext) -> Result<Self, Self::Error> {
        let snapshot = storage::instance().snapshot()?;
        let plugins = PluginManager::setup(&snapshot)?;

        Ok(Context {
            snapshot,
            plugins,
            storage: storage::instance(),
            // TODO(k82cn): Add ActionManager for them.
            actions: vec![
                AllocateAction::new_ptr(),
                ShuffleAction::new_ptr(),
                BackfillAction::new_ptr(),
            ],
            schedule_interval: DEFAULT_SCHEDULE_INTERVAL,
        })
    }
}

impl Context {
    pub fn filter(
        &self,
        execs: &Vec<ExecutorInfoPtr>,
        ssn: &SessionInfoPtr,
    ) -> Vec<ExecutorInfoPtr> {
        match lock_ptr!(self.plugins) {
            Ok(plugins) => plugins.filter(execs, ssn),
            Err(e) => {
                log::error!("Failed to lock plugin manager: {}", e);
                vec![]
            }
        }
    }

    pub fn filter_one(&self, exec: &ExecutorInfoPtr, ssn: &SessionInfoPtr) -> bool {
        !self.filter(&vec![exec.clone()], ssn).is_empty()
    }

    pub fn is_underused(&self, ssn: &SessionInfoPtr) -> bool {
        match lock_ptr!(self.plugins) {
            Ok(plugins) => plugins.is_underused(ssn),
            Err(e) => {
                log::error!("Failed to lock plugin manager: {}", e);
                true
            }
        }
    }

    pub fn is_preemptible(&self, ssn: &SessionInfoPtr) -> bool {
        match lock_ptr!(self.plugins) {
            Ok(plugins) => plugins.is_preemptible(ssn),
            Err(e) => {
                log::error!("Failed to lock plugin manager: {}", e);
                true
            }
        }
    }

    pub fn bind_session(
        &mut self,
        exec: &ExecutorInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<(), FlameError> {
        self.storage.bind_session(exec.id.clone(), ssn.id)?;

        {
            let mut plugins = lock_ptr!(self.plugins)?;
            plugins.on_session_bind(ssn);
        }

        self.snapshot
            .update_executor_state(exec.clone(), ExecutorState::Binding);

        Ok(())
    }

    pub fn pipeline_session(
        &mut self,
        exec: &ExecutorInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<(), FlameError> {
        {
            let mut plugins = lock_ptr!(self.plugins)?;
            plugins.on_session_bind(ssn);
        }

        self.snapshot
            .update_executor_state(exec.clone(), ExecutorState::Binding);

        Ok(())
    }

    pub fn unbind_session(
        &mut self,
        exec: &ExecutorInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<(), FlameError> {
        self.storage.unbind_executor(exec.id.clone())?;

        {
            let mut plugins = lock_ptr!(self.plugins)?;
            plugins.on_session_unbind(ssn);
        }

        self.snapshot
            .update_executor_state(exec.clone(), ExecutorState::Unbinding);

        Ok(())
    }
}
