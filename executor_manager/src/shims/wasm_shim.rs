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

use common::apis::{Application, SessionContext, TaskContext, TaskOutput};
use std::sync::{Arc, Mutex};

use crate::shims::{Shim, ShimPtr};
use common::FlameError;

#[derive(Clone)]
pub struct WasmShim {
    session_context: Option<SessionContext>,
}

impl WasmShim {
    pub fn new_ptr(_: &Application) -> ShimPtr {
        Arc::new(Mutex::new(Self {
            session_context: None,
        }))
    }
}

impl Shim for WasmShim {
    fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        todo!();
    }

    fn on_task_invoke(&mut self, ctx: &TaskContext) -> Result<Option<TaskOutput>, FlameError> {
        todo!();
    }

    fn on_session_leave(&mut self) -> Result<(), FlameError> {
        todo!();
    }
}
