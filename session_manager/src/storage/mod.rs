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

use crate::model::{FlameError, Session, SessionID, Task, TaskID};

mod memory;

pub fn new() -> Result<Box<dyn Storage>, FlameError> {
    Err(FlameError::NotFound("mem".to_string()))
}

#[async_trait]
pub trait Storage {
    async fn create_session(service_type: String, slots: i32) -> Result<Session, FlameError>;
    async fn get_session(id: SessionID) -> Result<Session, FlameError>;
    async fn delete_session(id: SessionID) -> Result<(), FlameError>;
    async fn update_session(ssn: &Session) -> Result<Session, FlameError>;
    async fn find_session() -> Result<Vec<Session>, FlameError>;

    async fn create_task(id: SessionID, task_input: &String) -> Result<Task, FlameError>;
    async fn get_task(ssn_id: SessionID, id: TaskID) -> Result<Task, FlameError>;
    async fn delete_task(ssn_id: SessionID, id: TaskID) -> Result<(), FlameError>;
    async fn update_task(t: &Task) -> Result<Task, FlameError>;
}
