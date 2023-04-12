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

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::ops::Deref;

use rpc::flame;

use std::sync::{Arc, LockResult, Mutex};

mod errors;
pub use crate::model::errors::FlameError;

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

#[derive(Debug, Default)]
pub struct Session {
    pub id: SessionID,
    pub application: String,
    pub slots: i32,
    pub tasks: Vec<Arc<Task>>,
    pub tasks_index: HashMap<TaskState, Vec<Arc<Task>>>,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub status: SessionStatus,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        let mut tasks = vec![];
        let mut tasks_index = HashMap::new();

        for t in &self.tasks {
            let task = Arc::new((*(*t)).clone());
            tasks.push(task.clone());
            match tasks_index.get_mut(&task.state) {
                None => {
                    tasks_index.insert(task.state, vec![task.clone()]);
                }
                Some(ts) => {
                    ts.push(task.clone());
                }
            };
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
