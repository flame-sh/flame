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

use async_trait::async_trait;

use crate::client::BackendClient;
use crate::executor::Executor;

use common::apis::ExecutorState;
use common::FlameError;

mod bound;
mod idle;
mod unbinding;
mod unknown;
mod void;

pub fn from(client: BackendClient, e: Executor) -> Box<dyn State> {
    log::info!("Build state <{}> for Executor <{}>.", e.state, e.id);

    match e.state {
        ExecutorState::Void => Box::new(void::VoidState {
            client,
            executor: e,
        }),
        ExecutorState::Idle => Box::new(idle::IdleState {
            client,
            executor: e,
        }),
        ExecutorState::Bound => Box::new(bound::BoundState {
            client,
            executor: e,
        }),
        ExecutorState::Unbinding => Box::new(unbinding::UnbindingState {
            client,
            executor: e,
        }),
        _ => Box::new(unknown::UnknownState { executor: e }),
    }
}

#[async_trait]
pub trait State: Send + Sync {
    async fn execute(&mut self) -> Result<Executor, FlameError>;
}
