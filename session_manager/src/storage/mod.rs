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

use crate::model::{Executor, ExecutorID, FlameError, Session, SessionID, Task, TaskID};

mod memory;

pub fn new() -> Result<Box<dyn Storage>, FlameError> {
    Err(FlameError::NotFound("mem".to_string()))
}

pub struct SnapShot {
    pub sessions: Vec<Session>,
    pub executors: Vec<Executor>,
}

#[async_trait]
pub trait Storage {
    async fn snapshot(&self) -> Result<SnapShot, FlameError>;

    async fn create_session(&self, service_type: String, slots: i32) -> Result<Session, FlameError>;
    async fn get_session(&self, id: SessionID) -> Result<Session, FlameError>;
    async fn delete_session(&self, id: SessionID) -> Result<(), FlameError>;
    async fn update_session(&self, ssn: &Session) -> Result<Session, FlameError>;
    async fn find_session(&self) -> Result<Vec<Session>, FlameError>;

    async fn create_task(&self, id: SessionID, task_input: &String) -> Result<Task, FlameError>;
    async fn get_task(&self, ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError>;
    async fn delete_task(&self, ssn_id: SessionID, id: TaskID) -> Result<(), FlameError>;
    async fn update_task(&self, t: &Task) -> Result<Task, FlameError>;

    async fn register_executor(&self, e: &Executor) -> Result<(), FlameError>;
    async fn get_executor(&self, id: ExecutorID) -> Result<Executor, FlameError>;
    async fn unregister_executor(&self, id: ExecutorID) -> Result<(), FlameError>;
    async fn update_executor(&self, e: &Executor) -> Result<Executor, FlameError>;
}
