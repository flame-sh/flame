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
        let plugins = PluginManager::setup(&snapshot.clone())?;

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
        self.plugins.filter(execs, ssn)
    }

    pub fn filter_one(&self, exec: &ExecutorInfoPtr, ssn: &SessionInfoPtr) -> bool {
        !self.filter(&vec![exec.clone()], ssn).is_empty()
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
        self.storage.bind_session(exec.id.clone(), ssn.id).await?;
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
        self.storage.unbind_executor(exec.id.clone()).await?;
        self.plugins.on_session_unbind(ssn)?;
        self.snapshot
            .update_executor_state(exec.clone(), ExecutorState::Unbinding)?;

        Ok(())
    }
}
