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
use std::ops::Deref;
use std::sync::Arc;

use common::apis::{
    Application, ApplicationID, CommonData, Executor, ExecutorID, ExecutorPtr, Session, SessionID,
    SessionPtr, SessionState, Task, TaskGID, TaskID, TaskInput, TaskOutput, TaskPtr, TaskState,
};
use common::ptr::{self, MutexPtr};
use common::{lock_ptr, FlameError};

use crate::model::{ExecutorInfo, SessionInfo, SnapShot, SnapShotPtr};
use crate::storage::engine::EnginePtr;

mod engine;

pub type StoragePtr = Arc<Storage>;

#[derive(Clone)]
pub struct Storage {
    engine: EnginePtr,
    sessions: MutexPtr<HashMap<SessionID, SessionPtr>>,
    executors: MutexPtr<HashMap<ExecutorID, ExecutorPtr>>,
}

pub async fn new_ptr(url: &str) -> Result<StoragePtr, FlameError> {
    Ok(Arc::new(Storage {
        engine: engine::connect(url).await?,
        sessions: ptr::new_ptr(HashMap::new()),
        executors: ptr::new_ptr(HashMap::new()),
    }))
}

impl Storage {
    pub fn snapshot(&self) -> Result<SnapShotPtr, FlameError> {
        let res = SnapShot::new();

        {
            let ssn_map = lock_ptr!(self.sessions)?;
            for ssn in ssn_map.deref().values() {
                let ssn = lock_ptr!(ssn)?;
                let info = SessionInfo::from(&(*ssn));
                res.add_session(Arc::new(info))?;
            }
        }

        {
            let exe_map = lock_ptr!(self.executors)?;
            for exe in exe_map.deref().values() {
                let exe = lock_ptr!(exe)?;
                let info = ExecutorInfo::from(&(*exe).clone());
                res.add_executor(Arc::new(info))?;
            }
        }

        Ok(Arc::new(res))
    }

    pub async fn load_data(&self) -> Result<(), FlameError> {
        let ssn_list = self.engine.find_session().await?;
        for ssn in ssn_list {
            let task_list = self.engine.find_tasks(ssn.id).await?;
            let mut ssn = ssn.clone();
            for task in task_list {
                let task = match task.state {
                    TaskState::Running => self.engine.retry_task(task.gid()).await?,
                    _ => task,
                };

                ssn.update_task(&task);
            }

            let mut ssn_map = lock_ptr!(self.sessions)?;
            ssn_map.insert(ssn.id, SessionPtr::new(ssn.into()));
        }

        Ok(())
    }

    pub async fn create_session(
        &self,
        app: String,
        slots: i32,
        common_data: Option<CommonData>,
    ) -> Result<Session, FlameError> {
        let ssn = self.engine.create_session(app, slots, common_data).await?;

        let mut ssn_map = lock_ptr!(self.sessions)?;
        ssn_map.insert(ssn.id, SessionPtr::new(ssn.clone().into()));

        Ok(ssn)
    }

    pub async fn close_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let ssn = self.engine.close_session(id).await?;

        let ssn_ptr = self.get_session_ptr(ssn.id)?;
        let mut ssn = lock_ptr!(ssn_ptr)?;
        ssn.status.state = SessionState::Closed;

        Ok(ssn.clone())
    }

    pub fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let ssn_ptr = self.get_session_ptr(id)?;
        let ssn = lock_ptr!(ssn_ptr)?;
        Ok(ssn.clone())
    }

    pub fn get_session_ptr(&self, id: SessionID) -> Result<SessionPtr, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn = ssn_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(ssn.clone())
    }

    pub fn get_task_ptr(&self, gid: TaskGID) -> Result<TaskPtr, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;
        let ssn_ptr = ssn_map
            .get(&gid.ssn_id)
            .ok_or(FlameError::NotFound(gid.ssn_id.to_string()))?;

        let ssn = lock_ptr!(ssn_ptr)?;
        let task_ptr = ssn
            .tasks
            .get(&gid.task_id)
            .ok_or(FlameError::NotFound(gid.to_string()))?;

        Ok(task_ptr.clone())
    }

    pub async fn delete_session(&self, id: SessionID) -> Result<Session, FlameError> {
        let ssn = self.engine.delete_session(id).await?;

        let mut ssn_map = lock_ptr!(self.sessions)?;
        ssn_map.remove(&ssn.id);

        Ok(ssn)
    }

    pub fn list_session(&self) -> Result<Vec<Session>, FlameError> {
        let mut ssn_list = vec![];
        let ssn_map = lock_ptr!(self.sessions)?;

        for ssn in ssn_map.deref().values() {
            let ssn = lock_ptr!(ssn)?;
            ssn_list.push((*ssn).clone());
        }

        Ok(ssn_list)
    }

    pub async fn create_task(
        &self,
        ssn_id: SessionID,
        task_input: Option<TaskInput>,
    ) -> Result<Task, FlameError> {
        let task = self.engine.create_task(ssn_id, task_input).await?;

        let ssn = self.get_session_ptr(ssn_id)?;
        let mut ssn = lock_ptr!(ssn)?;
        ssn.update_task(&task);

        Ok(task)
    }

    pub fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        let ssn_map = lock_ptr!(self.sessions)?;

        let ssn = ssn_map
            .get(&ssn_id)
            .ok_or(FlameError::NotFound(ssn_id.to_string()))?;

        let ssn = lock_ptr!(ssn)?;
        let task = ssn
            .tasks
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;
        let task = lock_ptr!(task)?;
        Ok(task.clone())
    }

    pub async fn get_application(&self, id: ApplicationID) -> Result<Application, FlameError> {
        self.engine.get_application(id).await
    }

    pub async fn list_application(&self) -> Result<Vec<Application>, FlameError> {
        self.engine.find_application().await
    }

    pub async fn update_task(
        &self,
        ssn: SessionPtr,
        task: TaskPtr,
        state: TaskState,
        output: Option<TaskOutput>,
    ) -> Result<(), FlameError> {
        let gid = TaskGID {
            ssn_id: {
                let ssn_ptr = lock_ptr!(ssn)?;
                ssn_ptr.id
            },
            task_id: {
                let task_ptr = lock_ptr!(task)?;
                task_ptr.id
            },
        };

        let task = self.engine.update_task(gid, state, output).await?;

        let mut ssn_ptr = lock_ptr!(ssn)?;
        ssn_ptr.update_task(&task);

        Ok(())
    }

    pub fn register_executor(&self, e: &Executor) -> Result<(), FlameError> {
        let mut exe_map = lock_ptr!(self.executors)?;
        let exe = ExecutorPtr::new(e.clone().into());
        exe_map.insert(e.id.clone(), exe);

        Ok(())
    }

    pub fn get_executor_ptr(&self, id: ExecutorID) -> Result<ExecutorPtr, FlameError> {
        let exe_map = lock_ptr!(self.executors)?;
        let exe = exe_map
            .get(&id)
            .ok_or(FlameError::NotFound(id.to_string()))?;

        Ok(exe.clone())
    }
}
