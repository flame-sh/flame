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

use futures::future::BoxFuture;

use crate::model::{ExecutorPtr, ExecutorState, SessionID, SessionPtr, Task, TaskPtr};
use crate::storage::states::{
    binding::BindingState, bound::BoundState, idle::IdleState, unbinding::UnbindingState,
};
use common::{lock_cond_ptr, FlameError};

mod binding;
mod bound;
mod idle;
mod unbinding;

pub fn from(exe_ptr: ExecutorPtr) -> Result<Box<dyn States>, FlameError> {
    let exe = lock_cond_ptr!(exe_ptr)?;
    log::debug!("Build state <{}> for Executor.", exe.state.to_string());

    match exe.state {
        ExecutorState::Idle => Ok(Box::new(IdleState {
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Binding => Ok(Box::new(BindingState {
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Bound => Ok(Box::new(BoundState {
            executor: exe_ptr.clone(),
        })),
        ExecutorState::Unbinding => Ok(Box::new(UnbindingState {
            executor: exe_ptr.clone(),
        })),
    }
}

pub trait States: Send + Sync + 'static {
    fn wait_for_session(&self) -> BoxFuture<'static, Result<SessionID, FlameError>>;

    fn bind_session(&self, ssn: SessionPtr) -> Result<(), FlameError>;
    fn bind_session_completed(&self) -> Result<(), FlameError>;

    fn unbind_executor(&self) -> Result<(), FlameError>;
    fn unbind_executor_completed(&self) -> Result<(), FlameError>;

    fn launch_task(&self, ssn: SessionPtr) -> Result<Option<Task>, FlameError>;
    fn complete_task(&self, ssn: SessionPtr, task: TaskPtr) -> Result<(), FlameError>;
}
