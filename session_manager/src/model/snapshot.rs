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
use std::rc::Rc;

use chrono::{DateTime, Utc};

use crate::model::{
    Application, Executor, ExecutorID, ExecutorState, Session, SessionID, SessionState, Task,
    TaskID, TaskState,
};

pub struct SnapShot {
    pub sessions: Vec<Rc<SessionInfo>>,
    pub ssn_index: HashMap<SessionID, Rc<SessionInfo>>,
    pub ssn_state_index: HashMap<SessionState, Vec<Rc<SessionInfo>>>,
    pub executors: Vec<Rc<ExecutorInfo>>,
    pub exec_index: HashMap<ExecutorID, Rc<ExecutorInfo>>,
    pub exec_state_index: HashMap<ExecutorState, Vec<Rc<ExecutorInfo>>>,
}

#[derive(Debug, Default, Clone)]
pub struct TaskInfo {
    pub id: TaskID,
    pub ssn_id: SessionID,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: TaskState,
}

#[derive(Debug, Default, Clone)]
pub struct SessionInfo {
    pub id: SessionID,
    pub application: String,
    pub slots: i32,

    pub tasks_status: HashMap<TaskState, i32>,
    pub executors: HashMap<ExecutorID, Rc<ExecutorInfo>>,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: SessionState,

    pub total: f64,
    pub desired: f64,
    pub allocated: f64,
}
//
// impl SessionInfo {
//     pub fn add_task_info(&mut self, task: &TaskInfo) {
//         let task = Rc::new((*task).clone());
//         if !self.tasks_index.contains_key(&task.state) {
//             self.tasks_index.insert(task.state.clone(), vec![]);
//         }
//         self.tasks.push(task.clone());
//         if let Some(ts) = self.tasks_index.get_mut(&task.state){
//              ts.insert(task.state.clone(), task);
//         }
//     }
//
//     pub fn delete_task_info(&mut self, task: &TaskInfo) {
//
//         if let Some(ts) = tasks_index.get_mut(&task.state){
//             tsts.remove;
//         }
//     }
//
//     pub fn update_task_info_state(&mut self, task: &TaskInfo, state: TaskState) {
//
//     }
//
//
// }

#[derive(Clone, Debug, Default)]
pub struct ExecutorInfo {
    pub id: ExecutorID,
    pub applications: Vec<AppInfo>,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}

#[derive(Clone, Debug, Default)]
pub struct AppInfo {
    pub name: String,
}

impl From<Application> for AppInfo {
    fn from(app: Application) -> Self {
        AppInfo::from(&app)
    }
}

impl From<&Application> for AppInfo {
    fn from(app: &Application) -> Self {
        AppInfo {
            name: app.name.to_string(),
        }
    }
}

impl From<&Executor> for ExecutorInfo {
    fn from(exec: &Executor) -> Self {
        let applications = exec.applications.iter().map(AppInfo::from).collect();

        ExecutorInfo {
            id: exec.id.clone(),
            applications,
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
        // let mut tasks = vec![];
        let mut tasks_status = HashMap::new();
        for (k, v) in &ssn.tasks_index {
            tasks_status.insert((*k).clone(), v.len() as i32);
        }

        SessionInfo {
            id: ssn.id,
            application: ssn.application.clone(),
            slots: ssn.slots,
            // tasks,
            tasks_status,
            executors: HashMap::new(),
            creation_time: ssn.creation_time.clone(),
            completion_time: ssn.completion_time.clone(),
            state: ssn.status.state,

            total: 0.0,
            desired: 0.0,
            allocated: 0.0,
        }
    }
}
