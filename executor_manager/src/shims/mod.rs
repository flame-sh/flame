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
mod stdio_shim;

use std::sync::{Arc, Mutex};

use crate::executor::{Application, SessionContext, TaskContext};
use common::ptr::MutexPtr;
use common::FlameError;
use log_shim::LogShim;

use self::stdio_shim::StdioShim;

pub type ShimPtr = MutexPtr<dyn Shim>;

pub fn from(_: &Application) -> Result<ShimPtr, FlameError> {
    // TODO(k82cn): Load shim based on application's configuration.
    Ok(Arc::new(Mutex::new(StdioShim {
        session_context: None,
        child: None,
    })))
}

pub trait Shim: Send + Sync + 'static {
    fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError>;
    fn on_task_invoke(&mut self, ctx: &TaskContext) -> Result<(), FlameError>;
    fn on_session_leave(&mut self) -> Result<(), FlameError>;
}
