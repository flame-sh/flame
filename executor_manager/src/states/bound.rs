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

use crate::client;
use crate::executor::{Executor, ExecutorState};
use crate::states::State;
use common::ctx::FlameContext;
use common::{trace::TraceFn, trace_fn, FlameError};

#[derive(Clone)]
pub struct BoundState {
    pub executor: Executor,
}

#[async_trait]
impl State for BoundState {
    async fn execute(&mut self, ctx: &FlameContext) -> Result<Executor, FlameError> {
        trace_fn!("BoundState::execute");

        let task = client::launch_task(ctx, &self.executor.clone()).await?;
        self.executor.task = task.clone();

        match task {
            Some(task_ctx) => {
                let shim_ptr = &mut self.executor.shim.clone().ok_or(FlameError::InvalidState(
                    "no shim in bound state".to_string(),
                ))?;
                {
                    let mut shim = shim_ptr.lock().await;
                    let output = shim.on_task_invoke(&task_ctx).await?;
                    if let Some(task_ctx) = &mut self.executor.task {
                        task_ctx.output = output;
                    }
                };

                client::complete_task(ctx, &self.executor.clone()).await?;

                let (ssn_id, task_id) = {
                    let task = &self.executor.task.clone().unwrap();
                    (task.session_id.clone(), task.task_id.clone())
                };
                log::debug!("Complete task <{}/{}>", ssn_id, task_id)
            }
            None => {
                self.executor.state = ExecutorState::Unbound;
            }
        }

        self.executor.task = None;

        Ok(self.executor.clone())
    }
}
