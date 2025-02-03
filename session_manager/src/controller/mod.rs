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

use std::collections::HashMap;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use common::apis::{
    Application, ApplicationID, CommonData, Executor, ExecutorID, ExecutorPtr, Session, SessionID,
    SessionPtr, SessionState, Task, TaskGID, TaskID, TaskInput, TaskOutput, TaskPtr, TaskState,
};

use common::ptr::{self, MutexPtr};
use common::{lock_ptr, trace::TraceFn, trace_fn, FlameError};

use crate::model::SnapShotPtr;
use crate::storage::StoragePtr;

mod states;

pub struct Controller {
    storage: StoragePtr,
}

pub type ControllerPtr = Arc<Controller>;

pub fn new_ptr(storage: StoragePtr) -> ControllerPtr {
    Arc::new(Controller { storage })
}

impl Controller {
    pub async fn create_session(
        &self,
        app: String,
        slots: i32,
        common_data: Option<CommonData>,
    ) -> Result<Session, FlameError> {
        self.storage.create_session(app, slots, common_data).await
    }

    pub async fn close_session(&self, id: SessionID) -> Result<Session, FlameError> {
        self.storage.close_session(id).await
    }

    pub fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        self.storage.get_session(id)
    }

    pub async fn delete_session(&self, id: SessionID) -> Result<Session, FlameError> {
       self.storage.delete_session(id).await
    }

    pub fn list_session(&self) -> Result<Vec<Session>, FlameError> {
        self.storage.list_session()
    }

    pub async fn create_task(
        &self,
        ssn_id: SessionID,
        task_input: Option<TaskInput>,
    ) -> Result<Task, FlameError> {
        self.storage.create_task(ssn_id, task_input).await
    }

    pub fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        self.storage.get_task(ssn_id, id)
    }



    pub async fn update_task(
        &self,
        ssn: SessionPtr,
        task: TaskPtr,
        state: TaskState,
        output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        self.storage.update_task(ssn, task, state, output).await
    }

    pub fn register_executor(&self, e: &Executor) -> Result<(), FlameError> {
        self.storage.register_executor(e)
    }
   
    pub fn snapshot(&self) -> Result<SnapShotPtr, FlameError> {
        self.storage.snapshot()
    }

    pub async fn get_application(&self, id: ApplicationID) -> Result<Application, FlameError> {
        self.storage.get_application(id).await
    }

    pub async fn list_application(&self) -> Result<Vec<Application>, FlameError> {
        self.storage.list_application().await
    }
}

impl Controller {
    pub async fn watch_task(&self, gid: TaskGID) -> Result<Task, FlameError> {
        let task_ptr = self.storage.get_task_ptr(gid)?;
        WatchTaskFuture::new(self.storage.clone(), &task_ptr)?.await?;

        let task = lock_ptr!(task_ptr)?;
        Ok((*task).clone())
    }

    pub async fn wait_for_session(&self, id: ExecutorID) -> Result<Session, FlameError> {
        trace_fn!("Controller::wait_for_session");
        let exe_ptr = self.storage.get_executor_ptr(id)?;
        let ssn_id = WaitForSsnFuture::new(&exe_ptr).await?;

        let ssn_ptr = self.storage.get_session_ptr(ssn_id)?;
        let ssn = lock_ptr!(ssn_ptr)?;

        Ok((*ssn).clone())
    }

    pub async fn bind_session(&self, id: ExecutorID, ssn_id: SessionID) -> Result<(), FlameError> {
        trace_fn!("Controller::bind_session");

        let exe_ptr = self.storage.get_executor_ptr(id)?;
        let state = states::from(self.storage.clone(), exe_ptr)?;

        let ssn_ptr = self.storage.get_session_ptr(ssn_id)?;
        state.bind_session(ssn_ptr).await?;

        Ok(())
    }

    pub async fn bind_session_completed(&self, id: ExecutorID) -> Result<(), FlameError> {
        trace_fn!("Controller::bind_session_completed");

        let exe_ptr = self.storage.get_executor_ptr(id)?;
        let state = states::from(self.storage.clone(), exe_ptr)?;

        state.bind_session_completed().await?;

        Ok(())
    }

    pub async fn launch_task(&self, id: ExecutorID) -> Result<Option<Task>, FlameError> {
        trace_fn!("Controller::launch_task");
        let exe_ptr = self.storage.get_executor_ptr(id)?;
        let state = states::from(self.storage.clone(), exe_ptr.clone())?;
        let (ssn_id, task_id) = {
            let exec = lock_ptr!(exe_ptr)?;
            (exec.ssn_id, exec.task_id)
        };
        let ssn_id = ssn_id.ok_or(FlameError::InvalidState(
            "no session in bound executor".to_string(),
        ))?;

        //
        if let Some(task_id) = task_id {
            log::warn!(
                "Re-launch the task <{}/{}>",
                ssn_id.clone(),
                task_id.clone()
            );
            let task_ptr = self.storage.get_task_ptr(TaskGID { ssn_id, task_id })?;

            let task = lock_ptr!(task_ptr)?;
            return Ok(Some((*task).clone()));
        }

        let ssn_ptr = self.storage.get_session_ptr(ssn_id)?;
        state.launch_task(ssn_ptr).await
    }

    pub async fn complete_task(
        &self,
        id: ExecutorID,
        task_output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        trace_fn!("Storage::complete_task");
        let exe_ptr = self.storage.get_executor_ptr(id)?;
        let (ssn_id, task_id) = {
            let exe = lock_ptr!(exe_ptr)?;
            (
                exe.ssn_id.ok_or(FlameError::InvalidState(
                    "no session in executor".to_string(),
                ))?,
                exe.task_id
                    .ok_or(FlameError::InvalidState("no task in executor".to_string()))?,
            )
        };

        let task_ptr = self.storage.get_task_ptr(TaskGID { ssn_id, task_id })?;
        let ssn_ptr = self.storage.get_session_ptr(ssn_id)?;

        let state = states::from(self.storage.clone(), exe_ptr)?;
        state.complete_task(ssn_ptr, task_ptr, task_output).await?;

        Ok(())
    }

    pub async fn unbind_executor(&self, id: ExecutorID) -> Result<(), FlameError> {
        let exe_ptr = self.storage.get_executor_ptr(id)?;
        let state = states::from(self.storage.clone(), exe_ptr)?;
        state.unbind_executor().await?;

        Ok(())
    }

    pub async fn unbind_executor_completed(&self, id: ExecutorID) -> Result<(), FlameError> {
        let exe_ptr = self.storage.get_executor_ptr(id)?;
        let state = states::from(self.storage.clone(), exe_ptr)?;

        state.unbind_executor_completed().await?;

        Ok(())
    }
}

struct WatchTaskFuture {
    storage: StoragePtr,
    current_state: TaskState,
    task_gid: TaskGID,
}

impl WatchTaskFuture {
    pub fn new(storage: StoragePtr, task_ptr: &TaskPtr) -> Result<Self, FlameError> {
        let task_ptr = task_ptr.clone();
        let task = lock_ptr!(task_ptr)?;

        Ok(Self {
            storage,
            current_state: task.state,
            task_gid: TaskGID {
                ssn_id: task.ssn_id,
                task_id: task.id,
            },
        })
    }
}

impl Future for WatchTaskFuture {
    type Output = Result<(), FlameError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let task_ptr = self.storage.get_task_ptr(self.task_gid)?;

        let task = lock_ptr!(task_ptr)?;
        // If the state of task was updated, return ready.
        if self.current_state != task.state || task.is_completed() {
            return Poll::Ready(Ok(()));
        }

        ctx.waker().wake_by_ref();
        Poll::Pending
    }
}

struct WaitForSsnFuture {
    executor: ExecutorPtr,
}

impl WaitForSsnFuture {
    pub fn new(exe_ptr: &ExecutorPtr) -> Self {
        Self {
            executor: exe_ptr.clone(),
        }
    }
}

impl Future for WaitForSsnFuture {
    type Output = Result<SessionID, FlameError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let exe = lock_ptr!(self.executor)?;

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
