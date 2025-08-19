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

use std::sync::Arc;

use crate::controller::states::{
    binding::BindingState, bound::BoundState, idle::IdleState, unbinding::UnbindingState,
    void::VoidState,
};
use crate::storage::StoragePtr;

use common::apis::{ExecutorPtr, ExecutorState, SessionPtr, Task, TaskOutput, TaskPtr};
use common::{lock_ptr, FlameError};

mod binding;
mod bound;
mod idle;
mod unbinding;
mod void;

pub fn from(storage: StoragePtr, exe_ptr: ExecutorPtr) -> Result<Arc<dyn States>, FlameError> {
    let exe = lock_ptr!(exe_ptr)?;
    log::debug!("Build state <{}> for Executor.", exe.state);

    match exe.state {
        ExecutorState::Void => Ok(Arc::new(VoidState {
            storage,
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Idle => Ok(Arc::new(IdleState {
            storage,
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Binding => Ok(Arc::new(BindingState {
            storage,
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Bound => Ok(Arc::new(BoundState {
            storage,
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Unbinding => Ok(Arc::new(UnbindingState {
            storage,
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Unknown => Err(FlameError::InvalidState("Executor is unknown".to_string())),

        ExecutorState::Releasing => Err(FlameError::InvalidState(
            "Executor is releasing".to_string(),
        )),
        ExecutorState::Released => {
            Err(FlameError::InvalidState("Executor is released".to_string()))
        }
    }
}

#[async_trait::async_trait]
pub trait States: Send + Sync + 'static {
    async fn register_executor(&self, exe: ExecutorPtr) -> Result<(), FlameError>;

    async fn bind_session(&self, ssn: SessionPtr) -> Result<(), FlameError>;
    async fn bind_session_completed(&self) -> Result<(), FlameError>;

    async fn unbind_executor(&self) -> Result<(), FlameError>;
    async fn unbind_executor_completed(&self) -> Result<(), FlameError>;

    async fn launch_task(&self, ssn: SessionPtr) -> Result<Option<Task>, FlameError>;
    async fn complete_task(
        &self,
        ssn: SessionPtr,
        task: TaskPtr,
        task_output: Option<TaskOutput>,
    ) -> Result<(), FlameError>;
}
