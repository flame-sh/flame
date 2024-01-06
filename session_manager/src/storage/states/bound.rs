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

use common::apis::{ExecutorPtr, ExecutorState, SessionPtr, Task, TaskOutput, TaskPtr, TaskState};
use common::{lock_ptr, trace::TraceFn, trace_fn, FlameError};

use crate::storage::states::States;
use crate::storage::StoragePtr;

pub struct BoundState {
    pub storage: StoragePtr,
    pub executor: ExecutorPtr,
}

#[async_trait::async_trait]
impl States for BoundState {
    async fn bind_session(&self, _ssn_ptr: SessionPtr) -> Result<(), FlameError> {
        todo!()
    }

    async fn bind_session_completed(&self) -> Result<(), FlameError> {
        todo!()
    }

    async fn unbind_executor(&self) -> Result<(), FlameError> {
        trace_fn!("BoundState::unbind_session");

        let mut e = lock_ptr!(self.executor)?;
        e.state = ExecutorState::Unbinding;

        Ok(())
    }

    async fn unbind_executor_completed(&self) -> Result<(), FlameError> {
        todo!()
    }

    async fn launch_task(&self, ssn_ptr: SessionPtr) -> Result<Option<Task>, FlameError> {
        trace_fn!("BoundState::launch_task");
        let task_ptr = {
            let mut ssn = lock_ptr!(ssn_ptr)?;
            ssn.pop_pending_task()
        };

        let task_ptr = {
            match task_ptr {
                Some(task_ptr) => {
                    self.storage
                        .update_task_state(ssn_ptr.clone(), task_ptr.clone(), TaskState::Running)
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

    async  fn complete_task(
        &self,
        ssn_ptr: SessionPtr,
        task_ptr: TaskPtr,
        task_output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        trace_fn!("BoundState::complete_task");

        {
            let mut e = lock_ptr!(self.executor)?;
            e.task_id = None;
        };

        {
            let mut task = lock_ptr!(task_ptr)?;
            task.output = task_output;
        }

        self.storage
            .update_task_state(ssn_ptr, task_ptr, TaskState::Succeed)
            .await?;

        Ok(())
    }
}
