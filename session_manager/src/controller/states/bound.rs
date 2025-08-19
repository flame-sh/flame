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

use chrono::{DateTime, Duration, Utc};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::model::ExecutorPtr;
use common::apis::{ExecutorState, SessionPtr, Task, TaskOutput, TaskPtr, TaskState};
use common::{lock_ptr, trace::TraceFn, trace_fn, FlameError};

use crate::controller::states::States;
use crate::storage::StoragePtr;

pub struct BoundState {
    pub storage: StoragePtr,
    pub executor: ExecutorPtr,
}

#[async_trait::async_trait]
impl States for BoundState {
    async fn register_executor(&self, _exe: ExecutorPtr) -> Result<(), FlameError> {
        trace_fn!("BoundState::register_executor");

        Err(FlameError::InvalidState("Executor is bound".to_string()))
    }

    async fn bind_session(&self, _ssn_ptr: SessionPtr) -> Result<(), FlameError> {
        trace_fn!("BoundState::bind_session");

        Err(FlameError::InvalidState("Executor is bound".to_string()))
    }

    async fn bind_session_completed(&self) -> Result<(), FlameError> {
        trace_fn!("BoundState::bind_session_completed");

        Err(FlameError::InvalidState("Executor is bound".to_string()))
    }

    async fn unbind_executor(&self) -> Result<(), FlameError> {
        trace_fn!("BoundState::unbind_session");

        let mut e = lock_ptr!(self.executor)?;
        e.state = ExecutorState::Unbinding;

        Ok(())
    }

    async fn unbind_executor_completed(&self) -> Result<(), FlameError> {
        trace_fn!("BoundState::unbind_executor_completed");

        Err(FlameError::InvalidState("Executor is bound".to_string()))
    }

    async fn launch_task(&self, ssn_ptr: SessionPtr) -> Result<Option<Task>, FlameError> {
        trace_fn!("BoundState::launch_task");

        let app_name = {
            let ssn = lock_ptr!(ssn_ptr)?;
            ssn.application.clone()
        };

        let app_ptr = self.storage.get_application(app_name).await?;

        let task_ptr = WaitForTaskFuture::new(&ssn_ptr, app_ptr.delay_release).await?;

        let task_ptr = {
            match task_ptr {
                Some(task_ptr) => {
                    self.storage
                        .update_task(ssn_ptr.clone(), task_ptr.clone(), TaskState::Running, None)
                        .await?;
                    Some(task_ptr)
                }
                None => None,
            }
        };

        // No pending task, return.
        if task_ptr.is_none() {
            return Ok(None);
        }

        // let task_ptr = task_ptr.unwrap();
        let (ssn_id, task_id) = {
            let task_ptr = task_ptr.clone().unwrap();
            let task = lock_ptr!(task_ptr)?;
            (task.ssn_id, task.id)
        };

        log::debug!("Launching task <{}/{}>", ssn_id.clone(), task_id.clone());

        {
            let mut e = lock_ptr!(self.executor)?;
            e.task_id = Some(task_id);
            e.ssn_id = Some(ssn_id);
        };

        let task_ptr = task_ptr.unwrap();
        let task = lock_ptr!(task_ptr)?;
        Ok(Some((*task).clone()))
    }

    async fn complete_task(
        &self,
        ssn_ptr: SessionPtr,
        task_ptr: TaskPtr,
        task_output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        trace_fn!("BoundState::complete_task");

        self.storage
            .update_task(ssn_ptr, task_ptr, TaskState::Succeed, task_output)
            .await?;

        {
            let mut e = lock_ptr!(self.executor)?;
            e.task_id = None;
        };

        Ok(())
    }
}

struct WaitForTaskFuture {
    ssn: SessionPtr,
    delay_release: Duration,
    start_time: DateTime<Utc>,
}

impl WaitForTaskFuture {
    pub fn new(ssn: &SessionPtr, delay_release: Duration) -> Self {
        Self {
            ssn: ssn.clone(),
            delay_release,
            start_time: Utc::now(),
        }
    }
}

impl Future for WaitForTaskFuture {
    type Output = Result<Option<TaskPtr>, FlameError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut ssn = lock_ptr!(self.ssn)?;

        match ssn.pop_pending_task() {
            None => {
                let now = Utc::now();
                if now - self.start_time > self.delay_release {
                    // If the delay release is reached, return None.
                    Poll::Ready(Ok(None))
                } else {
                    // If the delay release is not reached, wait for the next poll.
                    ctx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Some(task_ptr) => Poll::Ready(Ok(Some(task_ptr))),
        }
    }
}
