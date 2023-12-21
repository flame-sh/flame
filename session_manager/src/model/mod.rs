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

use common::apis::{
    Application, Executor, ExecutorID, ExecutorState, Session, SessionID, SessionState, Task,
    TaskID, TaskState,
};

pub type SessionInfoPtr = Rc<SessionInfo>;
pub type ExecutorInfoPtr = Rc<ExecutorInfo>;

#[derive(Clone)]
pub struct SnapShot {
    pub sessions: HashMap<SessionID, SessionInfoPtr>,
    pub ssn_index: HashMap<SessionState, HashMap<SessionID, SessionInfoPtr>>,

    pub executors: HashMap<ExecutorID, ExecutorInfoPtr>,
    pub exec_index: HashMap<ExecutorState, HashMap<ExecutorID, ExecutorInfoPtr>>,
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

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: SessionState,
}

#[derive(Clone, Debug, Default)]
pub struct ExecutorInfo {
    pub id: ExecutorID,
    pub slots: i32,
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
            slots: exec.slots,
            applications,
            task_id: exec.task_id,
            ssn_id: exec.ssn_id,
            creation_time: exec.creation_time,
            state: exec.state,
        }
    }
}

impl From<&Task> for TaskInfo {
    fn from(task: &Task) -> Self {
        TaskInfo {
            id: task.id,
            ssn_id: task.ssn_id,
            creation_time: task.creation_time,
            completion_time: task.completion_time,
            state: task.state,
        }
    }
}

impl From<&Session> for SessionInfo {
    fn from(ssn: &Session) -> Self {
        // let mut tasks = vec![];
        let mut tasks_status = HashMap::new();
        for (k, v) in &ssn.tasks_index {
            tasks_status.insert(*k, v.len() as i32);
        }

        SessionInfo {
            id: ssn.id,
            application: ssn.application.clone(),
            slots: ssn.slots,
            // tasks,
            tasks_status,
            creation_time: ssn.creation_time,
            completion_time: ssn.completion_time,
            state: ssn.status.state,
        }
    }
}

impl SnapShot {
    pub fn add_session(&mut self, ssn: SessionInfoPtr) {
        self.sessions.insert(ssn.id, ssn.clone());
        self.ssn_index.entry(ssn.state).or_default();

        if let Some(ssn_list) = self.ssn_index.get_mut(&ssn.state) {
            ssn_list.insert(ssn.id, ssn.clone());
        }
    }

    #[allow(dead_code)]
    pub fn delete_session(&mut self, ssn: SessionInfoPtr) {
        self.sessions.remove(&ssn.id);

        for ssn_list in &mut self.ssn_index.values_mut() {
            ssn_list.remove(&ssn.id);
        }
    }

    #[allow(dead_code)]
    pub fn update_session(&mut self, ssn: SessionInfoPtr) {
        self.delete_session(ssn.clone());
        self.add_session(ssn);
    }

    pub fn add_executor(&mut self, exec: ExecutorInfoPtr) {
        self.executors.insert(exec.id.clone(), exec.clone());
        self.exec_index
            .entry(exec.state)
            .or_default();

        if let Some(exec_list) = self.exec_index.get_mut(&exec.state.clone()) {
            exec_list.insert(exec.id.clone(), exec.clone());
        }
    }

    pub fn delete_executor(&mut self, exec: ExecutorInfoPtr) {
        self.executors.remove(&exec.id);
        for exec_list in &mut self.exec_index.values_mut() {
            exec_list.remove(&exec.id);
        }
    }

    pub fn update_executor_state(&mut self, exec: ExecutorInfoPtr, state: ExecutorState) {
        let new_exec = Rc::new(ExecutorInfo {
            id: exec.id.clone(),
            slots: exec.slots,
            applications: exec.applications.to_vec(),
            task_id: exec.task_id,
            ssn_id: exec.ssn_id,
            creation_time: exec.creation_time,
            state,
        });

        self.delete_executor(new_exec.clone());
        self.add_executor(new_exec);
    }
}
