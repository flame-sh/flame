/*
Copyright 2023 The Flame Authors.
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
use std::fmt;

use ::rpc::flame::ApplicationSpec;
use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use rpc::flame as rpc;

use crate::ptr::MutexPtr;
use crate::FlameError;

pub type SessionID = i64;
pub type TaskID = i64;
pub type ExecutorID = String;
pub type ApplicationID = String;
pub type TaskPtr = MutexPtr<Task>;
pub type SessionPtr = MutexPtr<Session>;
pub type ExecutorPtr = MutexPtr<Executor>;

type Message = bytes::Bytes;
pub type TaskInput = Message;
pub type TaskOutput = Message;
pub type CommonData = Message;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum ApplicationState {
    #[default]
    Enabled = 0,
    Disabled = 1,
}

#[derive(Clone, Debug, Default, Copy)]
pub enum Shim {
    #[default]
    Log = 0,
    Stdio = 1,
    Wasm = 2,
}

#[derive(Clone, Debug, Default)]
pub struct Application {
    pub name: String,
    pub state: ApplicationState,
    pub creation_time: DateTime<Utc>,
    pub shim: Shim,
    pub url: Option<String>,
    pub command: Option<String>,
    pub arguments: Vec<String>,
    pub environments: Vec<String>,
    pub working_directory: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum SessionState {
    #[default]
    Open = 0,
    Closed = 1,
}

#[derive(Clone, Debug, Default)]
pub struct SessionStatus {
    pub state: SessionState,
}

#[derive(Debug, Default)]
pub struct Session {
    pub id: SessionID,
    pub application: String,
    pub slots: i32,
    pub common_data: Option<CommonData>,
    pub tasks: HashMap<TaskID, TaskPtr>,
    pub tasks_index: HashMap<TaskState, HashMap<TaskID, TaskPtr>>,
    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub status: SessionStatus,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum TaskState {
    #[default]
    Pending = 0,
    Running = 1,
    Succeed = 2,
    Failed = 3,
}

#[derive(Clone, Debug, Default, Copy)]
pub struct TaskGID {
    pub ssn_id: SessionID,
    pub task_id: TaskID,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskID,
    pub ssn_id: SessionID,
    pub input: Option<TaskInput>,
    pub output: Option<TaskOutput>,

    pub creation_time: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,

    pub state: TaskState,
}

impl Task {
    pub fn is_completed(&self) -> bool {
        self.state == TaskState::Succeed || self.state == TaskState::Failed
    }

    pub fn gid(&self) -> TaskGID {
        TaskGID {
            ssn_id: self.ssn_id,
            task_id: self.id,
        }
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum ExecutorState {
    #[default]
    Idle = 0,
    Binding = 1,
    Bound = 2,
    Unbinding = 3,
}

#[derive(Clone, Debug)]
pub struct Executor {
    pub id: ExecutorID,
    pub slots: i32,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}

#[derive(Clone, Debug)]
pub struct TaskContext {
    pub id: String,
    pub ssn_id: String,
    pub input: Option<TaskInput>,
    pub output: Option<TaskOutput>,
}

#[derive(Clone, Debug)]
pub struct SessionContext {
    pub ssn_id: String,
    pub application: ApplicationContext,
    pub slots: i32,
    pub common_data: Option<CommonData>,
}

#[derive(Clone, Debug)]
pub struct ApplicationContext {
    pub name: String,
    pub url: Option<String>,
    pub command: Option<String>,
    pub shim: Shim,
}

impl Session {
    pub fn is_closed(&self) -> bool {
        self.status.state == SessionState::Closed
    }

    pub fn update_task(&mut self, task: &Task) {
        let task_ptr = TaskPtr::new(task.clone().into());

        self.tasks.insert(task.id, task_ptr.clone());
        self.tasks_index.entry(task.state).or_default();
        self.tasks_index
            .get_mut(&task.state)
            .unwrap()
            .insert(task.id, task_ptr);
    }

    pub fn pop_pending_task(&mut self) -> Option<TaskPtr> {
        let pending_tasks = self.tasks_index.get_mut(&TaskState::Pending)?;
        if let Some((task_id, _)) = pending_tasks.clone().iter().next() {
            return pending_tasks.remove(task_id);
        }

        None
    }
}

impl Clone for Session {
    fn clone(&self) -> Self {
        let mut ssn = Session {
            id: self.id,
            application: self.application.clone(),
            slots: self.slots,
            common_data: self.common_data.clone(),
            tasks: HashMap::new(),
            tasks_index: HashMap::new(),
            creation_time: self.creation_time,
            completion_time: self.completion_time,
            status: self.status.clone(),
        };

        for (id, t) in &self.tasks {
            match t.lock() {
                Ok(t) => {
                    ssn.update_task(&t);
                }
                Err(_) => {
                    log::error!("Failed to lock task: <{}>, ignore it during clone.", id);
                }
            }
        }

        ssn
    }
}

impl TryFrom<rpc::Task> for TaskContext {
    type Error = FlameError;

    fn try_from(task: rpc::Task) -> Result<Self, Self::Error> {
        let metadata = task
            .metadata
            .ok_or(FlameError::InvalidConfig("metadata".to_string()))?;

        let spec = task
            .spec
            .ok_or(FlameError::InvalidConfig("spec".to_string()))?;

        Ok(TaskContext {
            id: metadata.id,
            ssn_id: spec.session_id.to_string(),
            input: spec.input.map(TaskInput::from),
            output: spec.output.map(TaskOutput::from),
        })
    }
}

impl TryFrom<rpc::Application> for ApplicationContext {
    type Error = FlameError;

    fn try_from(app: rpc::Application) -> Result<Self, Self::Error> {
        let metadata = app
            .metadata
            .ok_or(FlameError::InvalidConfig("metadata".to_string()))?;

        let spec = app
            .spec
            .ok_or(FlameError::InvalidConfig("spec".to_string()))?;

        Ok(ApplicationContext {
            name: metadata.name.clone(),
            url: spec.url.clone(),
            command: spec.command.clone(),
            shim: Shim::try_from(spec.shim)
                .map_err(|_| FlameError::InvalidConfig("shim".to_string()))?,
        })
    }
}

impl TryFrom<rpc::BindExecutorResponse> for SessionContext {
    type Error = FlameError;

    fn try_from(resp: rpc::BindExecutorResponse) -> Result<Self, Self::Error> {
        let app = resp
            .application
            .ok_or(FlameError::InvalidConfig("application".to_string()))?;
        let ssn = resp
            .session
            .ok_or(FlameError::InvalidConfig("session".to_string()))?;

        let metadata = ssn
            .metadata
            .ok_or(FlameError::InvalidConfig("metadata".to_string()))?;

        let spec = ssn
            .spec
            .ok_or(FlameError::InvalidConfig("spec".to_string()))?;

        let application = ApplicationContext::try_from(app)?;

        Ok(SessionContext {
            ssn_id: metadata.id,
            application,
            slots: spec.slots,
            common_data: spec.common_data.map(CommonData::from),
        })
    }
}

impl From<Task> for rpc::Task {
    fn from(task: Task) -> Self {
        rpc::Task::from(&task)
    }
}

impl From<&Task> for rpc::Task {
    fn from(task: &Task) -> Self {
        let metadata = Some(rpc::Metadata {
            id: task.id.to_string(),
            name: task.id.to_string(),
            owner: Some(task.ssn_id.to_string()),
        });

        let spec = Some(rpc::TaskSpec {
            session_id: task.ssn_id.to_string(),
            input: task.input.clone().map(TaskInput::into),
            output: task.output.clone().map(TaskOutput::into),
        });
        let status = Some(rpc::TaskStatus {
            state: task.state as i32,
            creation_time: task.creation_time.timestamp(),
            completion_time: task.completion_time.map(|s| s.timestamp()),
        });
        rpc::Task {
            metadata,
            spec,
            status,
        }
    }
}

impl From<Session> for rpc::Session {
    fn from(ssn: Session) -> Self {
        rpc::Session::from(&ssn)
    }
}

impl From<&Session> for rpc::Session {
    fn from(ssn: &Session) -> Self {
        let mut status = rpc::SessionStatus {
            state: ssn.status.state as i32,
            creation_time: ssn.creation_time.timestamp(),
            completion_time: ssn.completion_time.map(|s| s.timestamp()),
            failed: 0,
            pending: 0,
            running: 0,
            succeed: 0,
        };
        for (s, v) in &ssn.tasks_index {
            match s {
                TaskState::Pending => status.pending = v.len() as i32,
                TaskState::Running => status.running = v.len() as i32,
                TaskState::Succeed => status.succeed = v.len() as i32,
                TaskState::Failed => status.failed = v.len() as i32,
            }
        }

        rpc::Session {
            metadata: Some(rpc::Metadata {
                id: ssn.id.to_string(),
                name: ssn.id.to_string(),
                owner: None,
            }),
            spec: Some(rpc::SessionSpec {
                application: ssn.application.clone(),
                slots: ssn.slots,
                common_data: ssn.common_data.clone().map(CommonData::into),
            }),
            status: Some(status),
        }
    }
}

impl TryFrom<rpc::Application> for Application {
    type Error = FlameError;
    fn try_from(app: rpc::Application) -> Result<Self, Self::Error> {
        Application::try_from(&app)
    }
}

impl TryFrom<&rpc::Application> for Application {
    type Error = FlameError;
    fn try_from(app: &rpc::Application) -> Result<Self, Self::Error> {
        let metadata = app.metadata.clone().ok_or(FlameError::InvalidConfig(
            "application metadata is empty".to_string(),
        ))?;

        let spec = app.spec.clone().ok_or(FlameError::InvalidConfig(
            "application spec is empty".to_string(),
        ))?;

        let status = app.status.clone().ok_or(FlameError::InvalidConfig(
            "application status is empty".to_string(),
        ))?;

        Ok(Application {
            name: metadata.name.clone(),
            state: ApplicationState::from(status.state()),
            creation_time: DateTime::<Utc>::from_timestamp(status.creation_time, 0).ok_or(
                FlameError::InvalidState("invalid creation time".to_string()),
            )?,
            shim: Shim::try_from(spec.shim).unwrap_or(Shim::default()),
            url: spec.url.clone(),
            command: spec.command.clone(),
            arguments: spec.arguments.to_vec(),
            environments: spec.environments.to_vec(),
            working_directory: spec.working_directory.unwrap_or(String::default()),
        })
    }
}

impl From<Application> for rpc::Application {
    fn from(app: Application) -> Self {
        rpc::Application::from(&app)
    }
}

impl From<&Application> for rpc::Application {
    fn from(app: &Application) -> Self {
        let spec = Some(ApplicationSpec {
            shim: app.shim.into(),
            url: app.url.clone(),
            command: app.command.clone(),
            arguments: app.arguments.to_vec(),
            environments: app.environments.to_vec(),
            working_directory: Some(app.working_directory.clone()),
        });
        let metadata = Some(rpc::Metadata {
            id: app.name.clone(),
            name: app.name.clone(),
            owner: None,
        });

        let status = Some(rpc::ApplicationStatus {
            state: app.state.into(),
            creation_time: app.creation_time.timestamp(),
        });
        rpc::Application {
            metadata,
            spec,
            status,
        }
    }
}

impl From<ApplicationState> for rpc::ApplicationState {
    fn from(s: ApplicationState) -> Self {
        match s {
            ApplicationState::Disabled => Self::Disabled,
            ApplicationState::Enabled => Self::Enabled,
        }
    }
}

impl From<rpc::ApplicationState> for ApplicationState {
    fn from(s: rpc::ApplicationState) -> Self {
        match s {
            rpc::ApplicationState::Disabled => Self::Disabled,
            rpc::ApplicationState::Enabled => Self::Enabled,
        }
    }
}

impl TryFrom<i32> for ApplicationState {
    type Error = FlameError;
    fn try_from(s: i32) -> Result<Self, Self::Error> {
        let state = rpc::ApplicationState::try_from(s)
            .map_err(|_| FlameError::InvalidState("unknown application state".to_string()))?;
        Ok(Self::from(state))
    }
}
impl From<ApplicationState> for i32 {
    fn from(s: ApplicationState) -> Self {
        s as i32
    }
}

impl From<rpc::SessionState> for SessionState {
    fn from(s: rpc::SessionState) -> Self {
        match s {
            rpc::SessionState::Open => SessionState::Open,
            rpc::SessionState::Closed => SessionState::Closed,
        }
    }
}

impl From<SessionState> for rpc::SessionState {
    fn from(state: SessionState) -> Self {
        match state {
            SessionState::Open => rpc::SessionState::Open,
            SessionState::Closed => rpc::SessionState::Closed,
        }
    }
}
impl TryFrom<i32> for SessionState {
    type Error = FlameError;
    fn try_from(s: i32) -> Result<Self, Self::Error> {
        let state = rpc::SessionState::try_from(s)
            .map_err(|_| FlameError::InvalidState("invalid session state".to_string()))?;

        Ok(Self::from(state))
    }
}

impl From<SessionState> for i32 {
    fn from(s: SessionState) -> Self {
        s as i32
    }
}

impl From<rpc::TaskState> for TaskState {
    fn from(s: rpc::TaskState) -> Self {
        match s {
            rpc::TaskState::Pending => TaskState::Pending,
            rpc::TaskState::Running => TaskState::Running,
            rpc::TaskState::Succeed => TaskState::Succeed,
            rpc::TaskState::Failed => TaskState::Failed,
        }
    }
}

impl From<TaskState> for rpc::TaskState {
    fn from(state: TaskState) -> Self {
        match state {
            TaskState::Pending => rpc::TaskState::Pending,
            TaskState::Running => rpc::TaskState::Running,
            TaskState::Succeed => rpc::TaskState::Succeed,
            TaskState::Failed => rpc::TaskState::Failed,
        }
    }
}

impl TryFrom<i32> for TaskState {
    type Error = FlameError;
    fn try_from(s: i32) -> Result<Self, Self::Error> {
        let state = rpc::TaskState::try_from(s)
            .map_err(|_| FlameError::InvalidState("invalid task state".to_string()))?;

        Ok(Self::from(state))
    }
}

impl From<TaskState> for i32 {
    fn from(s: TaskState) -> Self {
        s as i32
    }
}

impl From<rpc::Shim> for Shim {
    fn from(s: rpc::Shim) -> Self {
        match s {
            rpc::Shim::Log => Self::Log,
            rpc::Shim::Stdio => Self::Stdio,
            rpc::Shim::Wasm => Self::Wasm,
        }
    }
}

impl From<Shim> for rpc::Shim {
    fn from(s: Shim) -> Self {
        match s {
            Shim::Log => Self::Log,
            Shim::Stdio => Self::Stdio,
            Shim::Wasm => Self::Wasm,
        }
    }
}

impl TryFrom<i32> for Shim {
    type Error = FlameError;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        let s = rpc::Shim::try_from(v)
            .map_err(|_| FlameError::InvalidState("unknown shim".to_string()))?;
        Ok(Self::from(s))
    }
}

impl From<Shim> for i32 {
    fn from(s: Shim) -> Self {
        s as i32
    }
}

impl fmt::Display for TaskGID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.ssn_id, self.task_id)
    }
}

fn default_work_dir() -> String {
    String::from("/tmp")
}
