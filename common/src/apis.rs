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
use std::{env, fmt};

use chrono::{DateTime, Duration, Utc};
use rustix::system;
use serde_json::json;

use ::rpc::flame::ApplicationSpec;
use rpc::flame as rpc;

use crate::ptr::MutexPtr;
use crate::FlameError;

pub const DEFAULT_MAX_INSTANCES: i32 = i32::MAX;
pub const DEFAULT_DELAY_RELEASE: Duration = Duration::seconds(60);

pub type SessionID = i64;
pub type TaskID = i64;
pub type ExecutorID = String;
pub type ApplicationID = String;
pub type TaskPtr = MutexPtr<Task>;
pub type SessionPtr = MutexPtr<Session>;
pub type NodePtr = MutexPtr<Node>;
pub type ApplicationPtr = MutexPtr<Application>;

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
    Shell = 3,
    Grpc = 4,
}

#[derive(Clone, Debug)]
pub struct ApplicationSchema {
    pub input: String,
    pub output: String,
    pub common_data: String,
}

impl Default for ApplicationSchema {
    fn default() -> Self {
        let default_schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "string"
        });

        Self {
            input: default_schema.to_string(),
            output: default_schema.to_string(),
            common_data: default_schema.to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Application {
    pub name: String,
    pub state: ApplicationState,
    pub creation_time: DateTime<Utc>,
    pub shim: Shim,
    pub image: Option<String>,
    pub description: Option<String>,
    pub labels: Vec<String>,
    pub command: Option<String>,
    pub arguments: Vec<String>,
    pub environments: HashMap<String, String>,
    pub working_directory: String,
    pub max_instances: i32,
    pub delay_release: Duration,
    pub schema: Option<ApplicationSchema>,
}

#[derive(Clone, Debug)]
pub struct ApplicationAttributes {
    pub shim: Shim,
    pub image: Option<String>,
    pub description: Option<String>,
    pub labels: Vec<String>,
    pub command: Option<String>,
    pub arguments: Vec<String>,
    pub environments: HashMap<String, String>,
    pub working_directory: String,
    pub max_instances: i32,
    pub delay_release: Duration,
    pub schema: Option<ApplicationSchema>,
}

impl Default for ApplicationAttributes {
    fn default() -> Self {
        Self {
            shim: Shim::default(),
            image: None,
            description: None,
            labels: vec![],
            command: None,
            arguments: vec![],
            environments: HashMap::new(),
            working_directory: "/tmp".to_string(),
            max_instances: DEFAULT_MAX_INSTANCES,
            delay_release: DEFAULT_DELAY_RELEASE,
            schema: Some(ApplicationSchema::default()),
        }
    }
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
    Unknown = 0,
    Void = 1,
    Idle = 2,
    Binding = 3,
    Bound = 4,
    Unbinding = 5,
    Releasing = 6,
    Released = 7,
}

#[derive(Clone, Debug)]
pub struct TaskContext {
    pub task_id: String,
    pub session_id: String,
    pub input: Option<TaskInput>,
    pub output: Option<TaskOutput>,
}

#[derive(Clone, Debug)]
pub struct SessionContext {
    pub session_id: String,
    pub application: ApplicationContext,
    pub slots: i32,
    pub common_data: Option<CommonData>,
}

#[derive(Clone, Debug)]
pub struct ApplicationContext {
    pub name: String,
    pub image: Option<String>,
    pub command: Option<String>,
    pub arguments: Vec<String>,
    pub environments: HashMap<String, String>,

    pub shim: Shim,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum NodeState {
    #[default]
    Unknown = 0,
    Ready = 1,
    NotReady = 2,
}

#[derive(Clone, Debug, Default)]
pub struct NodeInfo {
    pub arch: String,
    pub os: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceRequirement {
    pub cpu: u64,
    pub memory: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub name: String,
    pub capacity: ResourceRequirement,
    pub allocatable: ResourceRequirement,
    pub info: NodeInfo,
    pub state: NodeState,
}

impl Node {
    pub fn new() -> Self {
        let name = system::uname().nodename().to_string_lossy().to_string();
        let mut node = Node {
            name,
            state: NodeState::Ready,
            ..Default::default()
        };

        node.refresh();

        node
    }

    pub fn refresh(&mut self) {
        let sysinfo = system::sysinfo();
        let memory = sysinfo.totalram;
        let cpu = num_cpus::get() as u64;

        let capacity = ResourceRequirement { cpu, memory };
        let allocatable = capacity.clone();

        let info = NodeInfo {
            arch: env::consts::ARCH.to_string(),
            os: env::consts::OS.to_string(),
        };

        self.capacity = capacity;
        self.allocatable = allocatable;
        self.info = info;
    }
}

impl From<ResourceRequirement> for rpc::ResourceRequirement {
    fn from(req: ResourceRequirement) -> Self {
        Self {
            cpu: req.cpu,
            memory: req.memory,
        }
    }
}

impl From<rpc::ResourceRequirement> for ResourceRequirement {
    fn from(req: rpc::ResourceRequirement) -> Self {
        Self {
            cpu: req.cpu,
            memory: req.memory,
        }
    }
}

impl From<&str> for ResourceRequirement {
    fn from(s: &str) -> Self {
        Self::from(&s.to_string())
    }
}

impl From<&String> for ResourceRequirement {
    fn from(s: &String) -> Self {
        let parts = s.split(',');
        let mut cpu = 0;
        let mut memory = 0;
        for p in parts {
            let mut parts = p.split('=').map(|s| s.trim());
            let key = parts.next();
            let value = parts.next();
            match (key, value) {
                (Some("cpu"), Some(value)) => cpu = value.parse::<u64>().unwrap_or(0),
                (Some("memory"), Some(value)) => memory = Self::parse_memory(value),
                (Some("mem"), Some(value)) => memory = Self::parse_memory(value),
                _ => {
                    log::error!("Invalid resource requirement: {s}");
                }
            }
        }
        Self { cpu, memory }
    }
}

impl ResourceRequirement {
    pub fn new(slots: i32, unit: &ResourceRequirement) -> Self {
        Self {
            cpu: slots as u64 * unit.cpu,
            memory: slots as u64 * unit.memory,
        }
    }

    pub fn to_slots(&self, unit: &ResourceRequirement) -> u64 {
        (self.cpu / unit.cpu).min(self.memory / unit.memory)
    }

    fn parse_memory(s: &str) -> u64 {
        let s = s.to_lowercase();
        let v = s[..s.len() - 1].parse::<u64>().unwrap_or(0);
        let unit = s[s.len() - 1..].to_string();
        // TODO(k82cn): return error if the unit is not valid.
        match unit.as_str() {
            "k" => v * 1024,
            "m" => v * 1024 * 1024,
            "g" => v * 1024 * 1024 * 1024,
            _ => s.parse::<u64>().unwrap_or(0),
        }
    }
}

impl From<NodeInfo> for rpc::NodeInfo {
    fn from(info: NodeInfo) -> Self {
        Self {
            arch: info.arch,
            os: info.os,
        }
    }
}

impl From<rpc::NodeInfo> for NodeInfo {
    fn from(info: rpc::NodeInfo) -> Self {
        Self {
            arch: info.arch,
            os: info.os,
        }
    }
}

impl From<NodeState> for rpc::NodeState {
    fn from(state: NodeState) -> Self {
        match state {
            NodeState::Unknown => rpc::NodeState::Unknown,
            NodeState::Ready => rpc::NodeState::Ready,
            NodeState::NotReady => rpc::NodeState::NotReady,
        }
    }
}

impl From<rpc::NodeState> for NodeState {
    fn from(state: rpc::NodeState) -> Self {
        match state {
            rpc::NodeState::Unknown => NodeState::Unknown,
            rpc::NodeState::Ready => NodeState::Ready,
            rpc::NodeState::NotReady => NodeState::NotReady,
        }
    }
}

impl From<NodeState> for i32 {
    fn from(state: NodeState) -> Self {
        match state {
            NodeState::Unknown => 0,
            NodeState::Ready => 1,
            NodeState::NotReady => 2,
        }
    }
}

impl From<i32> for NodeState {
    fn from(state: i32) -> Self {
        match state {
            0 => NodeState::Unknown,
            1 => NodeState::Ready,
            2 => NodeState::NotReady,
            _ => NodeState::Unknown,
        }
    }
}

impl From<Node> for rpc::Node {
    fn from(node: Node) -> Self {
        let status = Some(rpc::NodeStatus {
            state: node.state.into(),
            capacity: Some(node.capacity.into()),
            allocatable: Some(node.allocatable.into()),
            info: Some(node.info.into()),
        });

        Self {
            metadata: Some(rpc::Metadata {
                id: node.name.clone(),
                name: node.name.clone(),
                owner: None,
            }),
            spec: Some(rpc::NodeSpec {}),
            status,
        }
    }
}

impl From<rpc::Node> for Node {
    fn from(node: rpc::Node) -> Self {
        let status = node.status.unwrap_or_default();
        let metadata = node.metadata.unwrap_or_default();
        Self {
            name: metadata.name,
            capacity: status.capacity.unwrap_or_default().into(),
            allocatable: status.allocatable.unwrap_or_default().into(),
            info: status.info.unwrap_or_default().into(),
            state: status.state.into(),
        }
    }
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
                    log::error!("Failed to lock task: <{id}>, ignore it during clone.");
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
            task_id: metadata.id.clone(),
            session_id: spec.session_id.to_string(),
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
            image: spec.image.clone(),
            command: spec.command.clone(),
            arguments: spec.arguments.clone(),
            environments: spec
                .environments
                .clone()
                .into_iter()
                .map(|e| (e.name, e.value))
                .collect(),
            shim: Shim::try_from(spec.shim)
                .map_err(|_| FlameError::InvalidConfig("shim".to_string()))?,
        })
    }
}

impl From<TaskContext> for rpc::TaskContext {
    fn from(ctx: TaskContext) -> Self {
        Self {
            task_id: ctx.task_id.clone(),
            session_id: ctx.session_id.clone(),
            input: ctx.input.map(|d| d.into()),
        }
    }
}

impl From<SessionContext> for rpc::SessionContext {
    fn from(ctx: SessionContext) -> Self {
        Self {
            session_id: ctx.session_id.clone(),
            application: Some(ctx.application.into()),
            common_data: ctx.common_data.map(|d| d.into()),
        }
    }
}

impl From<ApplicationContext> for rpc::ApplicationContext {
    fn from(ctx: ApplicationContext) -> Self {
        Self {
            name: ctx.name.clone(),
            image: ctx.image.clone(),
            shim: ctx.shim.into(),
            command: ctx.command.clone(),
        }
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
            session_id: metadata.id,
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

impl From<ApplicationSchema> for rpc::ApplicationSchema {
    fn from(schema: ApplicationSchema) -> Self {
        Self {
            input: schema.input,
            output: schema.output,
            common_data: schema.common_data,
        }
    }
}

impl From<rpc::ApplicationSchema> for ApplicationSchema {
    fn from(schema: rpc::ApplicationSchema) -> Self {
        Self {
            input: schema.input,
            output: schema.output,
            common_data: schema.common_data,
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

        let status = app.status.ok_or(FlameError::InvalidConfig(
            "application status is empty".to_string(),
        ))?;

        Ok(Application {
            name: metadata.name.clone(),
            state: ApplicationState::from(status.state()),
            creation_time: DateTime::<Utc>::from_timestamp(status.creation_time, 0).ok_or(
                FlameError::InvalidState("invalid creation time".to_string()),
            )?,
            shim: Shim::try_from(spec.shim).unwrap_or(Shim::default()),
            image: spec.image.clone(),
            description: spec.description.clone(),
            labels: spec.labels.clone(),
            command: spec.command.clone(),
            arguments: spec.arguments.to_vec(),
            environments: spec
                .environments
                .clone()
                .into_iter()
                .map(|e| (e.name, e.value))
                .collect(),
            working_directory: spec.working_directory.unwrap_or(String::default()),
            max_instances: spec.max_instances.unwrap_or(DEFAULT_MAX_INSTANCES),
            delay_release: spec
                .delay_release
                .map(Duration::seconds)
                .unwrap_or(DEFAULT_DELAY_RELEASE),
            schema: spec.schema.map(ApplicationSchema::from),
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
            image: app.image.clone(),
            description: app.description.clone(),
            labels: app.labels.clone(),
            command: app.command.clone(),
            arguments: app.arguments.to_vec(),
            environments: app
                .environments
                .clone()
                .into_iter()
                .map(|(k, v)| rpc::Environment { name: k, value: v })
                .collect(),
            working_directory: Some(app.working_directory.clone()),
            max_instances: Some(app.max_instances),
            delay_release: Some(app.delay_release.num_seconds()),
            schema: app.schema.clone().map(rpc::ApplicationSchema::from),
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

impl From<rpc::ApplicationSpec> for ApplicationAttributes {
    fn from(spec: rpc::ApplicationSpec) -> Self {
        Self {
            shim: spec.shim().into(),
            image: spec.image.clone(),
            description: spec.description.clone(),
            labels: spec.labels.clone(),
            command: spec.command.clone(),
            arguments: spec.arguments.clone(),
            environments: spec
                .environments
                .clone()
                .into_iter()
                .map(|e| (e.name, e.value))
                .collect(),
            working_directory: spec.working_directory.clone().unwrap_or_default(),
            max_instances: spec.max_instances.unwrap_or(DEFAULT_MAX_INSTANCES),
            delay_release: spec
                .delay_release
                .map(Duration::seconds)
                .unwrap_or(DEFAULT_DELAY_RELEASE),
            schema: spec.schema.map(ApplicationSchema::from),
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
            rpc::Shim::Shell => Self::Shell,
            rpc::Shim::Grpc => Self::Grpc,
        }
    }
}

impl From<Shim> for rpc::Shim {
    fn from(s: Shim) -> Self {
        match s {
            Shim::Log => Self::Log,
            Shim::Stdio => Self::Stdio,
            Shim::Wasm => Self::Wasm,
            Shim::Shell => Self::Shell,
            Shim::Grpc => Self::Grpc,
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

impl From<rpc::ExecutorState> for ExecutorState {
    fn from(s: rpc::ExecutorState) -> Self {
        match s {
            rpc::ExecutorState::ExecutorVoid => ExecutorState::Void,
            rpc::ExecutorState::ExecutorIdle => ExecutorState::Idle,
            rpc::ExecutorState::ExecutorBinding => ExecutorState::Binding,
            rpc::ExecutorState::ExecutorBound => ExecutorState::Bound,
            rpc::ExecutorState::ExecutorUnbinding => ExecutorState::Unbinding,
            rpc::ExecutorState::ExecutorReleasing => ExecutorState::Releasing,
            rpc::ExecutorState::ExecutorReleased => ExecutorState::Released,
            _ => ExecutorState::Unknown,
        }
    }
}

impl From<ExecutorState> for rpc::ExecutorState {
    fn from(s: ExecutorState) -> Self {
        match s {
            ExecutorState::Void => rpc::ExecutorState::ExecutorVoid,
            ExecutorState::Idle => rpc::ExecutorState::ExecutorIdle,
            ExecutorState::Binding => rpc::ExecutorState::ExecutorBinding,
            ExecutorState::Bound => rpc::ExecutorState::ExecutorBound,
            ExecutorState::Unbinding => rpc::ExecutorState::ExecutorUnbinding,
            ExecutorState::Releasing => rpc::ExecutorState::ExecutorReleasing,
            ExecutorState::Released => rpc::ExecutorState::ExecutorReleased,
            _ => rpc::ExecutorState::ExecutorUnknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resreq_from_string() {
        let cases = vec![
            ("cpu=1,mem=256", (1, 256)),
            ("cpu=1,mem=1k", (1, 1024)),
            ("cpu=1,memory=1m", (1, 1024 * 1024)),
            ("cpu=1,memory=1g", (1, 1024 * 1024 * 1024)),
        ];

        for (input, expected) in cases {
            let resreq = ResourceRequirement::from(input);
            assert_eq!(resreq.cpu, expected.0);
            assert_eq!(resreq.memory, expected.1);
        }
    }
}
