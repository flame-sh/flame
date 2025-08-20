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

use crate::client::BackendClient;
use crate::shims::ShimPtr;
use ::rpc::flame::{self as rpc, ExecutorSpec, ExecutorStatus, Metadata};

use crate::states;
use common::apis::{ExecutorState, ResourceRequirement, SessionContext, TaskContext};
use common::FlameError;
use common::{lock_ptr, trace::TraceFn, trace_fn};

#[derive(Clone)]
pub struct Executor {
    pub id: String,
    pub resreq: ResourceRequirement,
    pub node: String,
    pub session: Option<SessionContext>,
    pub task: Option<TaskContext>,

    pub shim: Option<ShimPtr>,

    pub state: ExecutorState,
}

pub type ExecutorPtr = Arc<Mutex<Executor>>;

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
            resreq: spec.resreq.unwrap().into(),
            node: spec.node.clone(),
            session: None,
            task: None,
            shim: None,
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
        let metadata = Some(Metadata {
            id: e.id.clone(),
            name: e.id.clone(),
            owner: None,
        });

        let spec = Some(ExecutorSpec {
            resreq: Some(e.resreq.clone().into()),
            node: e.node.clone(),
        });

        let status = Some(ExecutorStatus {
            state: rpc::ExecutorState::from(e.state).into(),
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
        log::info!(
            "Update executor <{}> from <{}> to <{}>",
            self.id,
            self.state,
            next.state
        );
        self.state = next.state;
        self.shim = next.shim.clone();
        self.session = next.session.clone();
        self.task = next.task.clone();
    }
}

pub fn start(client: BackendClient, executor: ExecutorPtr) {
    tokio::task::spawn(async move {
        loop {
            let exec = {
                let exec = lock_ptr!(executor);
                match exec {
                    Ok(exec) => exec.clone(),
                    Err(e) => {
                        log::error!("Failed to lock executor: {e}");
                        return;
                    }
                }
            };

            let mut state = states::from(client.clone(), exec.clone());
            match state.execute().await {
                Ok(next_state) => {
                    let mut exec = lock_ptr!(executor);
                    match exec {
                        Ok(mut exec) => {
                            exec.update(&next_state);
                        }
                        Err(e) => {
                            log::error!("Failed to lock executor: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to execute: {e}");
                }
            }
        }
    });
}
