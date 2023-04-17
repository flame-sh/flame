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

use crate::executor::{Application, Executor, ExecutorState};
use crate::states::State;
use crate::{client, shims, ExecutorPtr};
use common::{lock_cond_ptr, trace::TraceFn, trace_fn, FlameContext, FlameError};

pub struct IdleState {
    pub executor: Executor,
}

#[async_trait]
impl State for IdleState {
    async fn execute(&mut self, ctx: &FlameContext) -> Result<Executor, FlameError> {
        trace_fn!("IdleState::execute");

        let ssn = client::bind_executor(ctx, &self.executor.clone()).await?;

        let app = ctx.get_application(&ssn.application);
        match app {
            None => {
                log::error!("Failed to find Application in Executor.");
                Err(FlameError::NotFound(format!(
                    "Application <{}>",
                    &ssn.application
                )))
            }
            Some(app) => {
                let app = Application::from(&app);
                let mut shim_ptr = shims::from(&app)?;
                let shim = Arc::get_mut(&mut shim_ptr)
                    .ok_or(FlameError::Internal("shim ptr".to_string()))?;

                // TODO(k82cn): if on_session_enter failed, add retry limits.
                shim.on_session_enter(&ssn).await?;

                client::bind_executor_completed(ctx, &self.executor.clone()).await?;

                // Own the shim.
                self.executor.shim = Some(shim_ptr.clone());
                self.executor.session = Some(ssn);
                self.executor.state = ExecutorState::Bound;

                Ok(self.executor.clone())
            }
        }
    }
}
