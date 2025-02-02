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

use chrono::{DateTime, Utc};

use uuid::Uuid;

use crate::shims::ShimPtr;
use ::rpc::flame::{self as rpc, ExecutorSpec, Metadata};

use common::apis::{Application, SessionContext, TaskContext};
use common::ctx::FlameContext;
use common::FlameError;

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

#[derive(Clone)]
pub struct Executor {
    pub id: String,
    pub slots: i32,
    pub applications: Vec<Application>,

    pub session: Option<SessionContext>,
    pub task: Option<TaskContext>,

    pub shim: Option<ShimPtr>,

    pub start_time: DateTime<Utc>,
    pub state: ExecutorState,
}

impl From<&Executor> for rpc::Executor {
    fn from(e: &Executor) -> Self {
        let metadata = Some(Metadata {
            id: e.id.clone(),
            name: e.id.clone(),
            owner: None,
        });

        let spec = Some(ExecutorSpec { slots: e.slots });

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

impl Executor {
    pub fn update_state(&mut self, next: &Executor) {
        self.state = next.state;
        self.shim = next.shim.clone();
    }

    pub async fn from_context(ctx: &FlameContext, slots: Option<i32>) -> Result<Self, FlameError> {
        // let applications = ctx.applications.iter().map(Application::from).collect();

        let exec = Executor {
            id: Uuid::new_v4().to_string(),
            slots: slots.unwrap_or(1),
            applications: vec![],
            session: None,
            task: None,
            shim: None,
            start_time: Utc::now(),
            state: ExecutorState::Init,
        };

        Ok(exec)
    }
}
