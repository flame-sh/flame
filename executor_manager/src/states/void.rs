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
use crate::states::State;
use common::apis::ExecutorState;
use common::{trace::TraceFn, trace_fn, FlameError};

#[derive(Clone)]
pub struct VoidState {
    pub client: BackendClient,
    pub executor: Executor,
}

#[async_trait]
impl State for VoidState {
    async fn execute(&mut self) -> Result<Executor, FlameError> {
        trace_fn!("VoidState::execute");

        self.client
            .register_executor(&self.executor.clone())
            .await?;

        self.executor.state = ExecutorState::Idle;

        Ok(self.executor.clone())
    }
}
