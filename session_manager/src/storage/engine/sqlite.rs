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
use async_trait::async_trait;

use crate::FlameError;
use common::apis::{Session, SessionID, Task, TaskID};

use crate::storage::engine::Engine;

pub struct SqliteEngine {}

#[async_trait]
impl Engine for SqliteEngine {
    async fn persist_session(&self, _ssn: &Session) -> Result<(), FlameError> {
        todo!()
    }
    async fn get_session(&self, _id: SessionID) -> Result<Session, FlameError> {
        todo!()
    }
    async fn delete_session(&self, _id: SessionID) -> Result<(), FlameError> {
        todo!()
    }
    async fn update_session(&self, _ssn: &Session) -> Result<(), FlameError> {
        todo!()
    }
    async fn find_session(&self) -> Result<Vec<Session>, FlameError> {
        todo!()
    }

    async fn persist_task(&self, _task: &Task) -> Result<(), FlameError> {
        todo!()
    }
    async fn get_task(&self, _ssn_id: SessionID, _id: TaskID) -> Result<Task, FlameError> {
        todo!()
    }
    async fn delete_task(&self, _ssn_id: SessionID, _id: TaskID) -> Result<(), FlameError> {
        todo!()
    }
    async fn update_task(&self, _t: &Task) -> Result<(), FlameError> {
        todo!()
    }
}
