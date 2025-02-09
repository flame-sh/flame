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

mod grpc_shim;
mod log_shim;
mod shell_shim;
mod stdio_shim;
mod wasm_shim;

use std::sync::Arc;

use async_trait::async_trait;
use grpc_shim::GrpcShim;
use tokio::sync::Mutex;

use crate::svcmgr::ServiceManagerPtr;

use self::log_shim::LogShim;
use self::shell_shim::ShellShim;
use self::stdio_shim::StdioShim;
use self::wasm_shim::WasmShim;

use common::apis::{ApplicationContext, SessionContext, Shim as ShimType, TaskContext, TaskOutput};

use common::FlameError;

pub type ShimPtr = Arc<Mutex<dyn Shim>>;

pub async fn new(
    app: &ApplicationContext,
    service_manager: ServiceManagerPtr,
) -> Result<ShimPtr, FlameError> {
    match app.shim {
        ShimType::Stdio => Ok(StdioShim::new_ptr(app)),
        ShimType::Wasm => Ok(WasmShim::new_ptr(app).await?),
        ShimType::Shell => Ok(ShellShim::new_ptr(app)),
        ShimType::Grpc => Ok(GrpcShim::new_ptr(app, service_manager).await?),
        _ => Ok(LogShim::new_ptr(app)),
    }
}

#[async_trait]
pub trait Shim: Send + Sync + 'static {
    async fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError>;
    async fn on_task_invoke(&mut self, ctx: &TaskContext)
        -> Result<Option<TaskOutput>, FlameError>;
    async fn on_session_leave(&mut self) -> Result<(), FlameError>;
}
