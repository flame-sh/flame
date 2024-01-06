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

use crate::model::{ExecutorInfoPtr, SessionInfoPtr, SnapShotPtr};
use crate::scheduler::actions::{ActionPtr, AllocateAction, BackfillAction, ShuffleAction};
use crate::scheduler::plugins::{PluginManager, PluginManagerPtr};

use crate::storage::StoragePtr;

use common::apis::ExecutorState;

use common::FlameError;

const DEFAULT_SCHEDULE_INTERVAL: u64 = 500;

pub struct Context {
    pub snapshot: SnapShotPtr,
    pub storage: StoragePtr,
    pub actions: Vec<ActionPtr>,
    pub plugins: PluginManagerPtr,
    pub schedule_interval: u64,
}

impl Context {
    pub fn new(storage: StoragePtr) -> Result<Self, FlameError> {
        let snapshot = storage.snapshot()?;
        let plugins = PluginManager::setup(&snapshot.borrow())?;

        Ok(Context {
            snapshot,
            plugins,
            storage,
            // TODO(k82cn): Add ActionManager for them.
            actions: vec![
                AllocateAction::new_ptr(),
                ShuffleAction::new_ptr(),
                BackfillAction::new_ptr(),
            ],
            schedule_interval: DEFAULT_SCHEDULE_INTERVAL,
        })
    }

    pub fn filter(
        &self,
        execs: &Vec<ExecutorInfoPtr>,
        ssn: &SessionInfoPtr,
    ) -> Vec<ExecutorInfoPtr> {
        self.plugins.borrow().filter(execs, ssn)
    }

    pub fn filter_one(&self, exec: &ExecutorInfoPtr, ssn: &SessionInfoPtr) -> bool {
        !self.filter(&vec![exec.clone()], ssn).is_empty()
    }

    pub fn is_underused(&self, ssn: &SessionInfoPtr) -> bool {
        self.plugins.borrow().is_underused(ssn)
    }

    pub fn is_preemptible(&self, ssn: &SessionInfoPtr) -> bool {
        self.plugins.borrow().is_preemptible(ssn)
    }

    pub fn bind_session(
        &self,
        exec: &ExecutorInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<(), FlameError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| FlameError::Internal(e.to_string()))?;
        runtime.block_on(self.storage.bind_session(exec.id.clone(), ssn.id))?;

        self.plugins.borrow_mut().on_session_bind(ssn);
        self.snapshot
            .borrow_mut()
            .update_executor_state(exec.clone(), ExecutorState::Binding);

        Ok(())
    }

    pub fn pipeline_session(
        &self,
        exec: &ExecutorInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<(), FlameError> {
        self.plugins.borrow_mut().on_session_bind(ssn);

        self.snapshot
            .borrow_mut()
            .update_executor_state(exec.clone(), ExecutorState::Binding);

        Ok(())
    }

    pub fn unbind_session(
        &self,
        exec: &ExecutorInfoPtr,
        ssn: &SessionInfoPtr,
    ) -> Result<(), FlameError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| FlameError::Internal(e.to_string()))?;
        runtime.block_on(self.storage.unbind_executor(exec.id.clone()))?;

        self.plugins.borrow_mut().on_session_unbind(ssn);

        self.snapshot
            .borrow_mut()
            .update_executor_state(exec.clone(), ExecutorState::Unbinding);

        Ok(())
    }
}
