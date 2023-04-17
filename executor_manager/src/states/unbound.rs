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

use async_trait::async_trait;
use std::sync::Arc;

use crate::client;
use crate::executor::Executor;
use crate::states::State;
use common::{trace::TraceFn, trace_fn, FlameContext, FlameError};

pub struct UnboundState {
    pub executor: Executor,
}

#[async_trait]
impl State for UnboundState {
    async fn execute(&mut self, ctx: &FlameContext) -> Result<Executor, FlameError> {
        trace_fn!("UnboundState::execute");

        client::unbind_executor(ctx, &self.executor.clone()).await?;
        if let Some(shim_ptr) = &mut self.executor.shim {
            let shim =
                Arc::get_mut(shim_ptr).ok_or(FlameError::Internal("shim ptr".to_string()))?;
            shim.on_session_leave().await?;
            client::unbind_executor_completed(ctx, &self.executor.clone()).await?;
        }

        Ok(self.executor.clone())
    }
}
