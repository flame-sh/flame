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

use common::apis::{ExecutorPtr, ExecutorState, SessionID, SessionPtr, Task, TaskPtr};
use crate::storage::states::States;
use common::{lock_cond_ptr, trace::TraceFn, trace_fn, FlameError};

pub struct IdleState {
    pub executor: ExecutorPtr,
}

impl States for IdleState {
    fn wait_for_session(&self) -> BoxFuture<'static, Result<SessionID, FlameError>> {
        struct WaitForSsnFuture {
            executor: ExecutorPtr,
        }

        impl Future for WaitForSsnFuture {
            type Output = Result<SessionID, FlameError>;

            fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
                let exe = lock_cond_ptr!(self.executor)?;

                match exe.ssn_id {
                    None => {
                        // No bound session, trigger waker.
                        ctx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Some(ssn_id) => Poll::Ready(Ok(ssn_id)),
                }
            }
        }

        Box::pin(WaitForSsnFuture {
            executor: self.executor.clone(),
        })
    }

    fn bind_session(&self, ssn_ptr: SessionPtr) -> Result<(), FlameError> {
        trace_fn!("IdleState::bind_session");

        let ssn_id = {
            let ssn = lock_cond_ptr!(ssn_ptr)?;
            ssn.id
        };

        let mut e = lock_cond_ptr!(self.executor)?;
        e.ssn_id = Some(ssn_id);
        e.state = ExecutorState::Binding;

        Ok(())
    }

    fn bind_session_completed(&self) -> Result<(), FlameError> {
        todo!()
    }

    fn unbind_executor(&self) -> Result<(), FlameError> {
        todo!()
    }

    fn unbind_executor_completed(&self) -> Result<(), FlameError> {
        todo!()
    }

    fn launch_task(&self, _ssn: SessionPtr) -> Result<Option<Task>, FlameError> {
        todo!()
    }

    fn complete_task(
        &self,
        _ssn: SessionPtr,
        _task: TaskPtr,
        _: Option<String>,
    ) -> Result<(), FlameError> {
        todo!()
    }
}
