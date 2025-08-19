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
pub struct UnbindingState {
    pub client: BackendClient,
    pub executor: Executor,
}

#[async_trait]
impl State for UnbindingState {
    async fn execute(&mut self) -> Result<Executor, FlameError> {
        trace_fn!("UnbindingState::execute");

        self.client.unbind_executor(&self.executor.clone()).await?;
        let shim_ptr = &mut self.executor.shim.clone().ok_or(FlameError::InvalidState(
            "no shim in bound state".to_string(),
        ))?;

        {
            let mut shim = shim_ptr.lock().await;
            shim.on_session_leave().await?;
        }

        self.client
            .unbind_executor_completed(&self.executor.clone())
            .await?;

        self.executor.task = None;
        self.executor.session = None;
        self.executor.shim = None;

        // After unbound from session, the executor is idle now.
        self.executor.state = ExecutorState::Idle;

        Ok(self.executor.clone())
    }
}
