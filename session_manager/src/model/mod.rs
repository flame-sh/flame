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
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};

use rpc::flame as rpc;

use common::apis::{
    Application, ExecutorID, ExecutorState, Node, NodeState, ResourceRequirement, Session,
    SessionID, SessionState, Task, TaskID, TaskState,
};
use common::ptr::MutexPtr;
use common::{lock_ptr, FlameError};

pub type SessionInfoPtr = Arc<SessionInfo>;
pub type ExecutorInfoPtr = Arc<ExecutorInfo>;
pub type NodeInfoPtr = Arc<NodeInfo>;

#[derive(Clone)]
pub struct SnapShot {
    pub unit: ResourceRequirement,
    pub applications: MutexPtr<HashMap<String, AppInfo>>,

    pub sessions: MutexPtr<HashMap<SessionID, SessionInfoPtr>>,
    pub ssn_index: MutexPtr<HashMap<SessionState, HashMap<SessionID, SessionInfoPtr>>>,

    pub executors: MutexPtr<HashMap<ExecutorID, ExecutorInfoPtr>>,
    pub exec_index: MutexPtr<HashMap<ExecutorState, HashMap<ExecutorID, ExecutorInfoPtr>>>,

    pub nodes: MutexPtr<HashMap<String, NodeInfoPtr>>,
}

pub type SnapShotPtr = Arc<SnapShot>;

impl SnapShot {
    pub fn new(unit: ResourceRequirement) -> Self {
        SnapShot {
            unit,
            applications: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            ssn_index: Arc::new(Mutex::new(HashMap::new())),
            executors: Arc::new(Mutex::new(HashMap::new())),
            exec_index: Arc::new(Mutex::new(HashMap::new())),
            nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn debug(&self) -> Result<(), FlameError> {
        if log::log_enabled!(log::Level::Debug) {
            let ssn_num = {
                let ssns = lock_ptr!(self.sessions)?;
                ssns.len()
            };
            let exe_num = {
                let exes = lock_ptr!(self.executors)?;
                exes.len()
            };

            log::debug!("Session: <{ssn_num}>, Executor: <{exe_num}>");
        }

        Ok(())
    }
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
    pub node: String,
    pub resreq: ResourceRequirement,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}

#[derive(Clone, Debug, Default)]
pub struct NodeInfo {
    pub name: String,
    pub allocatable: ResourceRequirement,
    pub state: NodeState,
}

#[derive(Clone, Debug, Default)]
pub struct AppInfo {
    pub name: String,
    pub max_instances: i32,
    pub delay_release: Duration,
}

impl From<Application> for AppInfo {
    fn from(app: Application) -> Self {
        AppInfo::from(&app)
    }
}

impl From<&Node> for NodeInfo {
    fn from(node: &Node) -> Self {
        NodeInfo {
            name: node.name.clone(),
            allocatable: node.allocatable.clone(),
            state: node.state,
        }
    }
}

impl From<&Application> for AppInfo {
    fn from(app: &Application) -> Self {
        AppInfo {
            name: app.name.to_string(),
            max_instances: app.max_instances,
            delay_release: app.delay_release,
        }
    }
}

impl From<&Executor> for ExecutorInfo {
    fn from(exec: &Executor) -> Self {
        ExecutorInfo {
            id: exec.id.clone(),
            node: exec.node.clone(),
            resreq: exec.resreq.clone(),
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

pub struct SessionFilter {
    pub state: Option<SessionState>,
    pub ids: Vec<SessionID>,
}

pub const OPEN_SESSION: Option<SessionFilter> = Some(SessionFilter {
    state: Some(SessionState::Open),
    ids: vec![],
});

pub struct ExecutorFilter {
    pub state: Option<ExecutorState>,
    pub ids: Vec<ExecutorID>,
}

pub struct NodeFilter {
    pub state: Option<NodeState>,
    pub names: Vec<String>,
}

pub const ALL_NODE: Option<NodeFilter> = None;

pub const IDLE_EXECUTOR: Option<ExecutorFilter> = Some(ExecutorFilter {
    state: Some(ExecutorState::Idle),
    ids: vec![],
});

pub const BOUND_EXECUTOR: Option<ExecutorFilter> = Some(ExecutorFilter {
    state: Some(ExecutorState::Bound),
    ids: vec![],
});

pub const ALL_EXECUTOR: Option<ExecutorFilter> = None;

pub struct AppFilter {
    pub names: Vec<String>,
}

pub const ALL_APPLICATION: Option<AppFilter> = None;

impl SnapShot {
    pub fn find_nodes(
        &self,
        filter: Option<NodeFilter>,
    ) -> Result<HashMap<String, NodeInfoPtr>, FlameError> {
        match filter {
            Some(filter) => self.find_nodes_by_filter(filter),
            None => self.find_all_nodes(),
        }
    }

    fn find_nodes_by_filter(
        &self,
        filter: NodeFilter,
    ) -> Result<HashMap<String, NodeInfoPtr>, FlameError> {
        let mut nodes = HashMap::new();

        {
            let nodes_list = lock_ptr!(self.nodes)?;

            for name in filter.names {
                if let Some(node) = nodes_list.get(&name) {
                    nodes.insert(name, node.clone());
                } else {
                    log::warn!("Node <{name}> not found.");
                }
            }
        }

        Ok(nodes)
    }

    fn find_all_nodes(&self) -> Result<HashMap<String, NodeInfoPtr>, FlameError> {
        let mut nodes = HashMap::new();

        {
            let nodes_list = lock_ptr!(self.nodes)?;

            for node in nodes_list.values() {
                nodes.insert(node.name.clone(), node.clone());
            }
        }

        Ok(nodes)
    }

    pub fn find_applications(
        &self,
        filter: Option<AppFilter>,
    ) -> Result<HashMap<String, AppInfo>, FlameError> {
        match filter {
            Some(filter) => self.find_applications_by_filter(filter),
            None => self.find_all_applications(),
        }
    }

    fn find_applications_by_filter(
        &self,
        filter: AppFilter,
    ) -> Result<HashMap<String, AppInfo>, FlameError> {
        let mut appinfos = HashMap::new();

        {
            let apps = lock_ptr!(self.applications)?;

            for name in filter.names {
                if let Some(app) = apps.get(&name) {
                    appinfos.insert(name, app.clone());
                }
            }
        }

        Ok(appinfos)
    }

    fn find_all_applications(&self) -> Result<HashMap<String, AppInfo>, FlameError> {
        let mut appinfos = HashMap::new();

        {
            let mut apps = lock_ptr!(self.applications)?;

            for app in apps.values() {
                appinfos.insert(app.name.clone(), app.clone());
            }
        }

        Ok(appinfos)
    }

    pub fn find_sessions(
        &self,
        filter: Option<SessionFilter>,
    ) -> Result<HashMap<SessionID, SessionInfoPtr>, FlameError> {
        match filter {
            Some(filter) => self.find_sessions_by_filter(filter),
            None => self.find_all_sessions(),
        }
    }

    fn find_sessions_by_filter(
        &self,
        filter: SessionFilter,
    ) -> Result<HashMap<SessionID, SessionInfoPtr>, FlameError> {
        let mut ssns = HashMap::new();

        {
            let sessions = lock_ptr!(self.sessions)?;

            for id in filter.ids {
                if let Some(ssn) = sessions.get(&id) {
                    ssns.insert(id, ssn.clone());
                }
            }
        }

        {
            let ssn_index = lock_ptr!(self.ssn_index)?;
            if let Some(state) = filter.state {
                if let Some(ssn_list) = ssn_index.get(&state) {
                    for ssn in ssn_list.values() {
                        ssns.insert(ssn.id, ssn.clone());
                    }
                }
            }
        }

        Ok(ssns)
    }

    fn find_all_sessions(&self) -> Result<HashMap<SessionID, SessionInfoPtr>, FlameError> {
        let mut ssns = HashMap::new();

        {
            let sessions = lock_ptr!(self.sessions)?;

            for ssn in sessions.values() {
                ssns.insert(ssn.id, ssn.clone());
            }
        }

        Ok(ssns)
    }

    pub fn add_node(&self, node: NodeInfoPtr) -> Result<(), FlameError> {
        {
            let mut nodes = lock_ptr!(self.nodes)?;
            nodes.insert(node.name.clone(), node.clone());
        }

        Ok(())
    }

    pub fn add_session(&self, ssn: SessionInfoPtr) -> Result<(), FlameError> {
        {
            let mut sessions = lock_ptr!(self.sessions)?;
            sessions.insert(ssn.id, ssn.clone());
        }

        {
            let mut ssn_index = lock_ptr!(self.ssn_index)?;
            ssn_index.entry(ssn.state).or_default();

            if let Some(ssn_list) = ssn_index.get_mut(&ssn.state) {
                ssn_list.insert(ssn.id, ssn.clone());
            }
        }

        Ok(())
    }

    pub fn get_session(&self, id: &SessionID) -> Result<SessionInfoPtr, FlameError> {
        let sessions = lock_ptr!(self.sessions)?;
        match sessions.get(id) {
            Some(ptr) => Ok(ptr.clone()),
            None => Err(FlameError::NotFound(format!("session <{id}> not found"))),
        }
    }

    pub fn delete_session(&self, ssn: SessionInfoPtr) -> Result<(), FlameError> {
        {
            let mut sessions = lock_ptr!(self.sessions)?;
            sessions.remove(&ssn.id);
        }

        {
            let mut ssn_index = lock_ptr!(self.ssn_index)?;
            for ssn_list in &mut ssn_index.values_mut() {
                ssn_list.remove(&ssn.id);
            }
        }

        Ok(())
    }

    pub fn update_session(&self, ssn: SessionInfoPtr) -> Result<(), FlameError> {
        self.delete_session(ssn.clone())?;
        self.add_session(ssn)?;

        Ok(())
    }

    pub fn find_executors(
        &self,
        filter: Option<ExecutorFilter>,
    ) -> Result<HashMap<ExecutorID, ExecutorInfoPtr>, FlameError> {
        match filter {
            Some(filter) => self.find_executors_by_filter(filter),
            None => self.find_all_executors(),
        }
    }

    fn find_executors_by_filter(
        &self,
        filter: ExecutorFilter,
    ) -> Result<HashMap<ExecutorID, ExecutorInfoPtr>, FlameError> {
        let mut execs = HashMap::new();

        {
            let executors = lock_ptr!(self.executors)?;

            for id in filter.ids {
                if let Some(e) = executors.get(&id) {
                    execs.insert(id, e.clone());
                }
            }
        }

        {
            let exec_index = lock_ptr!(self.exec_index)?;
            if let Some(state) = filter.state {
                if let Some(exec_list) = exec_index.get(&state) {
                    for e in exec_list.values() {
                        execs.insert(e.id.clone(), e.clone());
                    }
                }
            }
        }

        Ok(execs)
    }

    fn find_all_executors(&self) -> Result<HashMap<ExecutorID, ExecutorInfoPtr>, FlameError> {
        let mut execs = HashMap::new();

        {
            let executors = lock_ptr!(self.executors)?;

            for e in executors.values() {
                execs.insert(e.id.clone(), e.clone());
            }
        }

        Ok(execs)
    }

    pub fn add_executor(&self, exec: ExecutorInfoPtr) -> Result<(), FlameError> {
        {
            let mut executors = lock_ptr!(self.executors)?;
            executors.insert(exec.id.clone(), exec.clone());
        }

        {
            let mut exec_index = lock_ptr!(self.exec_index)?;
            exec_index.entry(exec.state).or_default();

            if let Some(exec_list) = exec_index.get_mut(&exec.state.clone()) {
                exec_list.insert(exec.id.clone(), exec.clone());
            }
        }

        Ok(())
    }

    pub fn delete_executor(&self, exec: ExecutorInfoPtr) -> Result<(), FlameError> {
        {
            let mut executors = lock_ptr!(self.executors)?;
            executors.remove(&exec.id);
        }
        {
            let mut exec_index = lock_ptr!(self.exec_index)?;
            for exec_list in &mut exec_index.values_mut() {
                exec_list.remove(&exec.id);
            }
        }

        Ok(())
    }

    pub fn update_executor_state(
        &self,
        exec: ExecutorInfoPtr,
        state: ExecutorState,
    ) -> Result<(), FlameError> {
        let new_exec = Arc::new(ExecutorInfo {
            id: exec.id.clone(),
            node: exec.node.clone(),
            resreq: exec.resreq.clone(),
            task_id: exec.task_id,
            ssn_id: exec.ssn_id,
            creation_time: exec.creation_time,
            state,
        });

        self.delete_executor(new_exec.clone())?;
        self.add_executor(new_exec)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Executor {
    pub id: ExecutorID,
    pub node: String,
    pub resreq: ResourceRequirement,
    pub task_id: Option<TaskID>,
    pub ssn_id: Option<SessionID>,

    pub creation_time: DateTime<Utc>,
    pub state: ExecutorState,
}

pub type ExecutorPtr = MutexPtr<Executor>;

impl From<rpc::Executor> for Executor {
    fn from(e: rpc::Executor) -> Self {
        Executor::from(&e)
    }
}

impl From<&rpc::Executor> for Executor {
    fn from(e: &rpc::Executor) -> Self {
        let spec = e.spec.clone().unwrap();
        let status = e.status.unwrap();
        let metadata = e.metadata.clone().unwrap();

        let state = rpc::ExecutorState::try_from(status.state).unwrap().into();

        Executor {
            id: metadata.id.clone(),
            node: spec.node.clone(),
            resreq: spec.resreq.unwrap().into(),
            task_id: None,
            ssn_id: None,
            creation_time: Utc::now(),
            state,
        }
    }
}

impl From<Executor> for rpc::Executor {
    fn from(e: Executor) -> Self {
        rpc::Executor::from(&e)
    }
}

impl From<&Executor> for rpc::Executor {
    fn from(e: &Executor) -> Self {
        let metadata = Some(rpc::Metadata {
            id: e.id.clone(),
            name: e.id.clone(),
            owner: None,
        });

        let spec = Some(rpc::ExecutorSpec {
            resreq: Some(e.resreq.clone().into()),
            node: e.node.clone(),
        });

        let status = Some(rpc::ExecutorStatus {
            state: rpc::ExecutorState::from(e.state).into(),
        });

        rpc::Executor {
            metadata,
            spec,
            status,
        }
    }
}
