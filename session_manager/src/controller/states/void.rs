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

use crate::controller::states::States;
use crate::storage::StoragePtr;

use common::apis::{ExecutorPtr, ExecutorState, SessionPtr, Task, TaskOutput, TaskPtr, TaskState};
use common::{lock_ptr, trace::TraceFn, trace_fn, FlameError};

pub struct VoidState {
    pub storage: StoragePtr,
    pub executor: ExecutorPtr,
}

#[async_trait::async_trait]
impl States for VoidState {
    async fn register_executor(&self, exe: ExecutorPtr) -> Result<(), FlameError> {
        let id = {
            let e = lock_ptr!(exe)?;
            e.id.clone()
        };

        let mut e = lock_ptr!(exe)?;
        if e.id != id {
            return Err(FlameError::InvalidState("Executor ID mismatch".to_string()));
        }
        e.state = ExecutorState::Idle;

        Ok(())
    }

    async fn bind_session(&self, _ssn_ptr: SessionPtr) -> Result<(), FlameError> {
        trace_fn!("VoidState::bind_session");

        Err(FlameError::InvalidState("Executor is void".to_string()))
    }

    async fn bind_session_completed(&self) -> Result<(), FlameError> {
        trace_fn!("VoidState::bind_session_completed");

        Err(FlameError::InvalidState("Executor is void".to_string()))
    }

    async fn unbind_executor(&self) -> Result<(), FlameError> {
        trace_fn!("VoidState::unbind_executor");

        Err(FlameError::InvalidState("Executor is void".to_string()))
    }

    async fn unbind_executor_completed(&self) -> Result<(), FlameError> {
        trace_fn!("VoidState::unbind_executor_completed");

        Err(FlameError::InvalidState("Executor is void".to_string()))
    }

    async fn launch_task(&self, _ssn: SessionPtr) -> Result<Option<Task>, FlameError> {
        trace_fn!("VoidState::launch_task");

        Err(FlameError::InvalidState("Executor is void".to_string()))
    }

    async fn complete_task(
        &self,
        ssn_ptr: SessionPtr,
        task_ptr: TaskPtr,
        task_output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        trace_fn!("VoidState::complete_task");

        Err(FlameError::InvalidState("Executor is void".to_string()))
    }
}
