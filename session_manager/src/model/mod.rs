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

use chrono::{DateTime, Utc};

use common::ptr::CondPtr;
use common::{lock_cond_ptr, FlameError};

pub use crate::model::snapshot::{ExecutorInfo, SessionInfo, SnapShot, TaskInfo};

mod snapshot;

pub type SessionID = i64;
pub type TaskID = i64;
pub type ExecutorID = String;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum SessionState {
    #[default]
    Open = 0,
    Closed = 1,
}

#[derive(Clone, Debug, Default)]
pub struct SessionStatus {
    pub state: SessionState,
    // pub total: f64,
    // pub desired: f64,
    // pub allocated: f64,
}

pub type TaskPtr = CondPtr<Task>;
pub type SessionPtr = CondPtr<Session>;
pub type ExecutorPtr = CondPtr<Executor>;

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

impl Session {
    pub fn add_task(&mut self, task: &Task) {
        let task_ptr = TaskPtr::new(task.clone());

        self.tasks.insert(task.id, task_ptr.clone());
        if !self.tasks_index.contains_key(&task.state) {
            self.tasks_index.insert(task.state.clone(), HashMap::new());
        }
        self.tasks_index
            .get_mut(&task.state)
            .unwrap()
            .insert(task.id, task_ptr.clone());
    }

    pub fn update_task_state(
        &mut self,
        task_ptr: TaskPtr,
        state: TaskState,
    ) -> Result<(), FlameError> {
        let mut task = lock_cond_ptr!(task_ptr)?;
        match self.tasks_index.get_mut(&task.state) {
            None => {
                log::error!(
                    "Failed to find task <{}> in state map <{}>.",
                    task.id,
                    task.state.to_string()
                );

                return Err(FlameError::NotFound(format!(
                    "task <{}> in state map <{}>",
                    task.id,
                    task.state.to_string()
                )));
            }

            Some(index) => {
                index.remove(&task.id);
            }
        }

        self.tasks.remove(&task.id);

        task.state = state;
        self.add_task(&*task);

        Ok(())
    }
}

impl Clone for Session {
    fn clone(&self) -> Self {
        let mut ssn = Session {
            id: self.id,
            application: self.application.clone(),
            slots: self.slots,
            tasks: HashMap::new(),
            tasks_index: HashMap::new(),
            creation_time: self.creation_time.clone(),
            completion_time: self.completion_time.clone(),
            status: self.status.clone(),
        };

        for (id, t) in &self.tasks {
            match t.ptr.lock() {
                Ok(t) => {
                    ssn.add_task(&*t);
                }
                Err(_) => {
                    log::error!("Failed to lock task: <{}>, ignore it during clone.", id);
                }
            }
        }

        ssn
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum TaskState {
    #[default]
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

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum ExecutorState {
    #[default]
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
    pub slots: i32,
    pub applications: Vec<Application>,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}
