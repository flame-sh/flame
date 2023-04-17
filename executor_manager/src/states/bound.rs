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

use crate::executor::{Application, Executor, ExecutorState, TaskContext};
use crate::states::State;
use crate::{client, shims, ExecutorPtr};
use common::{lock_cond_ptr, trace::TraceFn, trace_fn, FlameContext, FlameError};

pub struct BoundState {
    pub executor: Executor,
}

#[async_trait]
impl State for BoundState {
    async fn execute(&mut self, ctx: &FlameContext) -> Result<Executor, FlameError> {
        trace_fn!("BoundState::execute");

        let task = client::launch_task(ctx, &self.executor.clone()).await?;
        match task {
            Some(t) => {
                let task_ctx = TaskContext::try_from(t)?;
                if let Some(shim) = &self.executor.shim {
                    shim.on_task_invoke(&task_ctx).await?;
                    client::complete_task(ctx, &self.executor.clone()).await?;
                }
            }
            None => {
                self.executor.state = ExecutorState::Unbound;
            }
        }

        Ok(self.executor.clone())
    }
}
