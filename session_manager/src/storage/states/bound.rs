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
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::model::{ExecutorPtr, ExecutorState, SessionID, SessionPtr, TaskPtr};
use crate::storage::states::States;
use common::{lock_cond_ptr, trace::TraceFn, trace_fn, FlameError};

pub struct BoundState {
    pub executor: ExecutorPtr,
}

impl States for BoundState {
    fn wait_for_session(&self) -> BoxFuture<'static, Result<SessionID, FlameError>> {
        todo!()
    }

    fn bind_session(&self, ssn_ptr: SessionPtr) -> Result<(), FlameError> {
        todo!()
    }

    fn bind_session_completed(&self) -> Result<(), FlameError> {
        todo!()
    }

    fn unbind_session(&self) -> Result<(), FlameError> {
        trace_fn!("BoundState::unbind_session");

        let mut e = lock_cond_ptr!(self.executor)?;
        e.state = ExecutorState::Unbinding;

        Ok(())
    }

    fn unbind_session_completed(&self) -> Result<(), FlameError> {
        todo!()
    }

    fn launch_task(&self, task: TaskPtr) -> Result<(), FlameError> {
        trace_fn!("BoundState::launch_task");

        let (task_id, ssn_id) = {
            let t = lock_cond_ptr!(task)?;
            (t.id, t.ssn_id)
        };

        let mut e = lock_cond_ptr!(self.executor)?;
        e.task_id = Some(task_id);
        e.ssn_id = Some(ssn_id);

        Ok(())
    }

    fn complete_task(&self, _task: TaskPtr) -> Result<(), FlameError> {
        trace_fn!("BoundState::complete_task");

        let mut e = lock_cond_ptr!(self.executor)?;
        e.task_id = None;
        e.ssn_id = None;

        Ok(())
    }
}
