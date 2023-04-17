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

use crate::executor::{SessionContext, TaskContext};
use crate::shims::Shim;
use common::FlameError;

pub struct LogShim {
    pub session_context: Option<SessionContext>,
}

#[async_trait]
impl Shim for LogShim {
    async fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        log::info!(
            "on_session_enter: Session: <{}>, Application: <{}>, Slots: <{}>",
            ctx.ssn_id,
            ctx.application,
            ctx.slots
        );
        self.session_context = Some(ctx.clone());

        Ok(())
    }

    async fn on_task_invoke(&mut self, ctx: &TaskContext) -> Result<(), FlameError> {
        log::info!(
            "on_task_invoke: Task: <{}>, Session: <{}>, Input: <{}>",
            ctx.id,
            ctx.ssn_id,
            ctx.input.unwrap_or("".to_string())
        );
        Ok(())
    }

    async fn on_session_leave(&mut self) -> Result<(), FlameError> {
        match &self.session_context {
            None => {
                log::info!("on_session_leave")
            }
            Some(ctx) => {
                log::info!(
                    "on_session_leave: Session: <{}>, Application: <{}>, Slots: <{}>",
                    ctx.ssn_id,
                    ctx.application,
                    ctx.slots
                );
            }
        }

        Ok(())
    }
}
