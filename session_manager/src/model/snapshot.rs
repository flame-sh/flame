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

use crate::lock_ptr;
use crate::model::{
    Application, Executor, ExecutorID, ExecutorState, Session, SessionID, SessionState, Task,
    TaskID, TaskState,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct SnapShot {
    pub sessions: Vec<SessionInfo>,
    pub executors: Vec<ExecutorInfo>,
}

#[derive(Debug, Default)]
pub struct TaskInfo {
    pub id: TaskID,
    pub ssn_id: SessionID,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: TaskState,
}

#[derive(Debug, Default)]
pub struct SessionInfo {
    pub id: SessionID,
    pub application: String,
    pub slots: i32,
    pub tasks: Vec<TaskInfo>,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: SessionState,

    pub total: f64,
    pub desired: f64,
    pub allocated: f64,
}

#[derive(Clone, Debug)]
pub struct ExecutorInfo {
    pub id: ExecutorID,
    pub application: Application,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}

impl From<&Executor> for ExecutorInfo {
    fn from(exec: &Executor) -> Self {
        ExecutorInfo {
            id: exec.id.clone(),
            application: exec.application.clone(),
            task_id: exec.task_id.clone(),
            ssn_id: exec.ssn_id.clone(),
            creation_time: exec.creation_time.clone(),
            state: exec.state,
        }
    }
}

impl From<&Task> for TaskInfo {
    fn from(task: &Task) -> Self {
        TaskInfo {
            id: task.id,
            ssn_id: task.ssn_id,
            creation_time: task.creation_time.clone(),
            completion_time: task.completion_time.clone(),
            state: task.state,
        }
    }
}

impl From<&Session> for SessionInfo {
    fn from(ssn: &Session) -> Self {
        let mut tasks = vec![];
        for (_, t) in &ssn.tasks {
            let task = t.lock();
            if task.is_err() {
                continue;
            }
            let task = task.unwrap().clone();
            tasks.push(TaskInfo::from(&task));
        }

        SessionInfo {
            id: ssn.id,
            application: ssn.application.clone(),
            slots: ssn.slots,
            tasks,
            creation_time: ssn.creation_time.clone(),
            completion_time: ssn.completion_time.clone(),
            state: ssn.status.state,

            total: ssn.status.total,
            desired: ssn.status.desired,
            allocated: ssn.status.allocated,
        }
    }
}
