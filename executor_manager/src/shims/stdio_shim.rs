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

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::executor::{Application, SessionContext, TaskContext};
use crate::shims::{Shim, ShimPtr};
use common::ptr::MutexPtr;
use common::{FlameError};

type ChildPtr = MutexPtr<Child>;

#[derive(Clone)]
pub struct StdioShim {
    application: Application,
    session_context: Option<SessionContext>,
}

impl StdioShim {
    pub fn new(app: &Application) -> ShimPtr {
        return Arc::new(Mutex::new(StdioShim {
            application: app.clone(),
            session_context: None,
        }));
    }
}

impl Shim for StdioShim {
    fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        self.session_context = Some(ctx.clone());

        Ok(())
    }

    fn on_task_invoke(&mut self, ctx: &TaskContext) -> Result<(), FlameError> {
        let mut child = Command::new(&self.application.command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(&self.application.working_directory)
            .spawn()
            .map_err(|_| FlameError::Internal("failed to start subprocess".to_string()))?;

        let mut stdin = child.stdin.take().unwrap();
        if let Some(input) = &ctx.input {
            let input = input.clone();
            let _handler = thread::spawn(move || {
                match stdin.write_all(input.as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Failed to send input into shim instance: {}.", e);
                    }
                };
            });
        }

        let mut stdout = child.stdout.take().unwrap();
        let mut data = String::new();
        let _n = stdout
            .read_to_string(&mut data)
            .map_err(|_| FlameError::Internal("failed to read task output".to_string()))?;

        log::debug!("The output is <{}>", data);

        Ok(())
    }

    fn on_session_leave(&mut self) -> Result<(), FlameError> {
        Ok(())
    }
}
