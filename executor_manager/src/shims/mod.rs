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

mod log_shim;

use async_trait::async_trait;

use crate::executor::{Application, SessionContext, TaskContext};
use common::FlameError;
use log_shim::LogShim;

pub fn from(_: &Application) -> Result<Box<dyn Shim>, FlameError> {
    // TODO(k82cn): Load shim based on application's configuration.
    Ok(Box::new(LogShim {}))
}

#[async_trait]
pub trait Shim: Send + Sync + 'static {
    async fn on_session_enter(&self, ctx: &SessionContext) -> Result<(), FlameError>;
    async fn on_task_invoke(&self, ctx: &TaskContext) -> Result<(), FlameError>;
    async fn on_session_leave(&self, ctx: &SessionContext) -> Result<(), FlameError>;
}
