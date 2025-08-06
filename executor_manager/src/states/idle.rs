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

use crate::executor::{Executor, ExecutorState};
use crate::states::State;
use crate::{client, shims};
use common::ctx::FlameContext;
use common::{trace::TraceFn, trace_fn, FlameError};

#[derive(Clone)]
pub struct IdleState {
    pub executor: Executor,
}

#[async_trait]
impl State for IdleState {
    async fn execute(&mut self, ctx: &FlameContext) -> Result<Executor, FlameError> {
        trace_fn!("IdleState::execute");

        let ssn = client::bind_executor(ctx, &self.executor.clone()).await?;

        log::debug!(
            "Try to bind Executor <{}> to <{}>.",
            &self.executor.id.clone(),
            &ssn.session_id.clone()
        );

        let shim_ptr = shims::new(&ssn.application).await?;
        {
            // TODO(k82cn): if on_session_enter failed, add retry limits.
            let mut shim = shim_ptr.lock().await;
            shim.on_session_enter(&ssn).await?;
            log::debug!("Shim on_session_enter completed.");
        };

        client::bind_executor_completed(ctx, &self.executor.clone()).await?;

        // Own the shim.
        self.executor.shim = Some(shim_ptr.clone());
        self.executor.session = Some(ssn.clone());
        self.executor.state = ExecutorState::Bound;

        log::debug!(
            "Executor <{}> was bound to <{}>.",
            &self.executor.id.clone(),
            &ssn.session_id.clone()
        );

        Ok(self.executor.clone())
    }
}
