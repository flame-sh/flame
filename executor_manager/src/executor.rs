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

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::shims::ShimPtr;
use ::rpc::flame::{self as rpc, ExecutorSpec, Metadata};
use crate::client::BackendClient;

use common::apis::{Application, SessionContext, TaskContext};
use common::apis::ResourceRequirement;
use common::ctx::FlameContext;
use common::lock_ptr;
use common::FlameError;
use crate::states;

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
    pub client: BackendClient,

    pub id: String,
    pub applications: Vec<Application>,
    pub resreq: ResourceRequirement,

    pub session: Option<SessionContext>,
    pub task: Option<TaskContext>,

    pub shim: Option<ShimPtr>,

    pub start_time: DateTime<Utc>,
    pub state: ExecutorState,
}

pub type ExecutorPtr = Arc<Mutex<Executor>>;

impl From<&Executor> for rpc::Executor {
    fn from(e: &Executor) -> Self {
        let metadata = Some(Metadata {
            id: e.id.clone(),
            name: e.id.clone(),
            owner: None,
        });

        let spec = Some(ExecutorSpec { resreq: Some(e.resreq.clone().into()) });

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
    pub fn update(&mut self, next: &Executor) {
        self.state = next.state;
        self.shim = next.shim.clone();
        self.session = next.session.clone();
        self.task = next.task.clone();
    }

    pub fn new(client: BackendClient, resreq: ResourceRequirement) -> Self {
        Self {
            client,
            resreq,
            id: Uuid::new_v4().to_string(),
            applications: vec![],
            session: None,
            task: None,
            shim: None,
            start_time: Utc::now(),
            state: ExecutorState::Init,
        }
    }
}

pub fn start(executor: ExecutorPtr) {
    tokio::task::spawn(async move {
        let exec = {
            let exec = lock_ptr!(executor)?;
            exec.clone()
        };
        let mut state = states::from(exec);
        match state.execute().await {
            Ok(next_state) => {
                let mut exec = lock_ptr!(executor)?;
                exec.update(&next_state);
            }
            Err(e) => {
                log::error!("Failed to execute: {e}");
            }
        }
    });
}
