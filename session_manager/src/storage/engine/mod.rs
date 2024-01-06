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

use std::sync::Arc;

use async_trait::async_trait;

use crate::FlameError;
use common::apis::{CommonData, Session, SessionID, Task, TaskID, TaskInput, TaskState};

mod sqlite;

pub type EnginePtr = Arc<dyn Engine>;

#[async_trait]
pub trait Engine: Send + Sync + 'static {
    async fn create_session(
        &self,
        app: String,
        slots: i32,
        common_data: Option<CommonData>,
    ) -> Result<Session, FlameError>;
    async fn get_session(&self, id: SessionID) -> Result<Session, FlameError>;
    async fn close_session(&self, id: SessionID) -> Result<Session, FlameError>;
    async fn delete_session(&self, id: SessionID) -> Result<Session, FlameError>;
    async fn find_session(&self) -> Result<Vec<Session>, FlameError>;

    async fn create_task(
        &self,
        ssn_id: SessionID,
        task_input: Option<TaskInput>,
    ) -> Result<Task, FlameError>;
    async fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError>;
    async fn delete_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError>;
    async fn update_task_state(
        &self,
        ssn_id: SessionID,
        id: TaskID,
        state: TaskState,
    ) -> Result<Task, FlameError>;
    async fn find_tasks(&self, ssn_id: SessionID) -> Result<Vec<Task>, FlameError>;
}

pub async fn connect() -> Result<EnginePtr, FlameError> {
    let url = String::from("sqlite://flame.db");

    Ok(sqlite::SqliteEngine::new_ptr(&url).await?)
}
