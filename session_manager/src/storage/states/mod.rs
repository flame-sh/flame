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

use common::{FlameError, lock_cond_ptr};
use crate::model::{ExecutorPtr, SessionID, SessionPtr, TaskPtr};
use crate::storage::states::idle::IdleState;

mod idle;

pub fn from(exe_ptr: ExecutorPtr) -> Result<Box<dyn States>, FlameError> {
    let exe = lock_cond_ptr!(exe_ptr)?;
    match exe.state {
        _ => Ok(Box::new(IdleState{executor: exe_ptr.clone()})),
    }
}

pub trait States {
    fn wait_for_session(&self) -> Result<SessionID, FlameError>;

    fn bind_session(&self, ssn: SessionPtr)-> Result<(), FlameError>;
    fn bind_session_completed(&self, ssn: SessionPtr)-> Result<(), FlameError>;

    fn unbind_session(&self) -> Result<(), FlameError>;
    fn unbind_session_completed(&self) -> Result<(), FlameError>;

    fn launch_task(&self, task: TaskPtr) -> Result<(), FlameError>;
    fn complete_task(&self, task: TaskPtr) -> Result<(), FlameError>;
}