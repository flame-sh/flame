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

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, LockResult, Mutex};

use chrono::{DateTime, Utc};

pub use crate::model::errors::FlameError;
use rpc::flame;

mod errors;
mod macros;

pub type SessionID = i64;
pub type TaskID = i64;
pub type ExecutorID = String;

#[derive(Clone, Copy, Debug, Default)]
pub enum SessionState {
    #[default]
    Open = 0,
    Closed = 1,
}

#[derive(Clone, Debug, Default)]
pub struct SessionStatus {
    pub state: SessionState,

    pub total: f64,
    pub desired: f64,
    pub allocated: f64,
}

type TaskPtr = Arc<Mutex<Task>>;
type SessionPtr = Arc<Mutex<Session>>;

#[derive(Debug, Default)]
pub struct Session {
    pub id: SessionID,
    pub application: String,
    pub slots: i32,
    pub tasks: HashMap<TaskID, TaskPtr>,
    pub tasks_index: HashMap<TaskState, HashMap<TaskID, TaskPtr>>,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub status: SessionStatus,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        let mut tasks: HashMap<TaskID, TaskPtr> = HashMap::new();
        let mut tasks_index: HashMap<TaskState, HashMap<TaskID, TaskPtr>> = HashMap::new();

        for (id, t) in &self.tasks {
            let t = t.lock();
            if t.is_err() {
                log::error!("Failed to lock task: <{}>, ignore it during clone.", id);
                continue;
            }
            let t = t.unwrap();
            let task = Arc::new(Mutex::new(t.clone()));

            tasks.insert(*id, task.clone());

            if !tasks_index.contains_key(&t.state) {
                tasks_index.insert(t.state, HashMap::new());
            }
            tasks_index
                .get_mut(&t.state)
                .unwrap()
                .insert(*id, task.clone());
        }

        Session {
            id: self.id,
            application: self.application.clone(),
            slots: self.slots,
            tasks,
            tasks_index,
            creation_time: self.creation_time.clone(),
            completion_time: self.completion_time.clone(),
            status: self.status.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TaskState {
    Pending = 0,
    Running = 1,
    Succeed = 2,
    Failed = 3,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskID,
    pub ssn_id: SessionID,
    pub input: Option<String>,
    pub output: Option<String>,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: TaskState,
}

#[derive(Clone, Copy, Debug)]
pub enum ExecutorState {
    Idle = 0,
    Binding = 1,
    Bound = 2,
    Unbinding = 3,
    Unknown = 4,
}

#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environments: Vec<String>,
    pub working_directory: String,
}

#[derive(Clone, Debug)]
pub struct Executor {
    pub id: ExecutorID,
    pub application: Application,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}
