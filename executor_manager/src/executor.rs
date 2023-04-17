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
use std::rc::Rc;
use std::sync::Arc;
use uuid::Uuid;

use crate::shims::Shim;
use ::rpc::flame as rpc;
use common::ptr::CondPtr;
use common::{FlameContext, FlameError};

use crate::states;

pub type ExecutorPtr = CondPtr<Executor>;

#[derive(Clone, Copy, Debug)]
pub enum ExecutorState {
    Init = 0,
    Idle = 1,
    Bound = 2,
    Unbound = 3,
    Unknown = 4,
}

impl From<ExecutorState> for rpc::ExecutorState {
    fn from(state: ExecutorState) -> Self {
        match state {
            ExecutorState::Init | ExecutorState::Idle => rpc::ExecutorState::ExecutorIdle,
            ExecutorState::Bound => rpc::ExecutorState::ExecutorBound,
            ExecutorState::Unbound => rpc::ExecutorState::ExecutorRunning,
            ExecutorState::Unknown => rpc::ExecutorState::ExecutorUnknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environments: Vec<String>,
    pub working_directory: String,
}

impl From<&Application> for rpc::Application {
    fn from(app: &Application) -> Self {
        rpc::Application {
            name: app.name.clone(),
            command: app.command.clone(),
            arguments: app.arguments.to_vec(),
            environments: app.environments.to_vec(),
            working_directory: app.working_directory.clone(),
        }
    }
}

impl From<&common::Application> for Application {
    fn from(app: &common::Application) -> Self {
        Application {
            name: app.name.to_string(),
            command: app.command_line.to_string(),
            arguments: vec![],
            environments: vec![],
            working_directory: app.working_directory.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaskContext {
    pub id: String,
    pub ssn_id: String,
    pub input: Option<String>,
    pub output: Option<String>,
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
            id: metadata.id.to_string(),
            ssn_id: spec.session_id.to_string(),
            input: spec.input.clone(),
            output: spec.output.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct SessionContext {
    pub ssn_id: String,
    pub application: String,
    pub slots: i32,
}

impl TryFrom<rpc::Session> for SessionContext {
    type Error = FlameError;

    fn try_from(ssn: rpc::Session) -> Result<Self, Self::Error> {
        let metadata = ssn
            .metadata
            .ok_or(FlameError::InvalidConfig("metadata".to_string()))?;
        let spec = ssn
            .spec
            .ok_or(FlameError::InvalidConfig("spec".to_string()))?;

        Ok(SessionContext {
            ssn_id: metadata.id.clone(),
            application: spec.application.clone(),
            slots: spec.slots,
        })
    }
}

#[derive(Clone)]
pub struct Executor {
    pub id: String,
    pub slots: i32,
    pub applications: Vec<Application>,

    pub session: Option<SessionContext>,
    pub task: Option<TaskContext>,

    pub shim: Option<Arc<dyn Shim>>,

    pub start_time: DateTime<Utc>,
    pub state: ExecutorState,
}

impl From<&Executor> for rpc::Executor {
    fn from(e: &Executor) -> Self {
        let metadata = Some(rpc::Metadata {
            id: e.id.clone(),
            owner: None,
        });

        let spec = Some(rpc::ExecutorSpec {
            slots: e.slots,
            applications: e.applications.iter().map(rpc::Application::from).collect(),
        });

        let status = Some(rpc::ExecutorStatus {
            state: rpc::ExecutorState::from(e.state) as i32,
        });

        rpc::Executor {
            metadata,
            spec,
            status,
        }
    }
}

impl From<&Executor> for rpc::ExecutorSpec {
    fn from(e: &Executor) -> Self {
        rpc::ExecutorSpec {
            slots: e.slots,
            applications: e.applications.iter().map(rpc::Application::from).collect(),
        }
    }
}

impl Executor {
    pub fn update_state(&mut self, next: &Executor) {
        self.state = next.state;
        self.shim = next.shim.clone();
    }

    pub async fn from_context(ctx: &FlameContext, slots: Option<i32>) -> Result<Self, FlameError> {
        let applications = ctx.applications.iter().map(Application::from).collect();

        let exec = Executor {
            id: Uuid::new_v4().to_string(),
            slots: slots.unwrap_or(1),
            applications,
            session: None,
            task: None,
            shim: None,
            start_time: Utc::now(),
            state: ExecutorState::Init,
        };

        Ok(exec)
    }
}
