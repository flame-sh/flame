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

use crate::FlameError;
use crate::model::{Executor, ExecutorID, Session, SessionID, Task, TaskID};
use crate::storage::{SnapShot, Storage};

pub struct NoneStorage {

}

impl Storage for NoneStorage {
    async fn snapshot(&self) -> Result<SnapShot, FlameError> {
        Ok(SnapShot{
            sessions: vec![],
            executors: vec![]
        })
    }

    async fn persist_session(&self, ssn: &Session) -> Result<(), FlameError> {
        Ok(())
    }

    async fn get_session(&self, id: SessionID) -> Result<Session, FlameError> {
        todo!()
    }

    async fn delete_session(&self, id: SessionID) -> Result<(), FlameError> {
        Ok(())
    }

    async fn update_session(&self, ssn: &Session) -> Result<(), FlameError> {
        Ok(())
    }

    async fn find_session(&self) -> Result<Vec<Session>, FlameError> {
        Ok(vec![])
    }

    async fn persist_task(&self, task: &Task) -> Result<(), FlameError> {
        Ok(())
    }

    async fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError> {
        todo!()
    }

    async fn delete_task(&self, ssn_id: SessionID, id: TaskID) -> Result<(), FlameError> {
        Ok(())
    }

    async fn update_task(&self, t: &Task) -> Result<(), FlameError> {
        Ok(())
    }

    async fn persist_executor(&self, e: &Executor) -> Result<(), FlameError> {
        Ok(())
    }

    async fn get_executor(&self, id: ExecutorID) -> Result<Executor, FlameError> {
        todo!()
    }

    async fn delete_executor(&self, id: ExecutorID) -> Result<(), FlameError> {
        Ok(())
    }

    async fn update_executor(&self, e: &Executor) -> Result<(), FlameError> {
        Ok(())
    }
}