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

use std::io::{Read, Write};
use std::path::{Path, MAIN_SEPARATOR};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::{env, thread};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::shims::{Shim, ShimPtr};
use common::apis::{Application, SessionContext, TaskContext, TaskOutput};
use common::FlameError;

const FLAME_TASK_ID: &str = "FLAME_TASK_ID";
const FLAME_SESSION_ID: &str = "FLAME_SESSION_ID";

#[derive(Clone)]
pub struct StdioShim {
    application: Application,
    session_context: Option<SessionContext>,
}

impl StdioShim {
    pub fn new_ptr(app: &Application) -> ShimPtr {
        Arc::new(Mutex::new(Self {
            application: app.clone(),
            session_context: None,
        }))
    }
}

#[async_trait]
impl Shim for StdioShim {
    async fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        self.session_context = Some(ctx.clone());

        Ok(())
    }

    async fn on_task_invoke(
        &mut self,
        ctx: &TaskContext,
    ) -> Result<Option<TaskOutput>, FlameError> {
        let mut cmd = self.application.command.clone();
        let path = Path::new(&cmd);
        if !path.has_root() {
            match env::current_dir() {
                Ok(cwd) => match cwd.to_str() {
                    None => {
                        log::warn!("Failed to get current directory path string.");
                    }
                    Some(cwd) => {
                        cmd = format!("{}{}{}", cwd, MAIN_SEPARATOR, cmd);
                    }
                },
                Err(e) => {
                    log::warn!(
                        "Failed to get current directory for application <{}>: {}",
                        &cmd,
                        e
                    );
                }
            }
        }

        let mut child = Command::new(&cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(&self.application.working_directory)
            .env(FLAME_TASK_ID, &ctx.id)
            .env(FLAME_SESSION_ID, &ctx.ssn_id)
            .spawn()
            .map_err(|_| FlameError::Internal("failed to start subprocess".to_string()))?;

        let mut stdin = child.stdin.take().unwrap();
        if let Some(input) = &ctx.input {
            let input = input.clone();
            let _handler = thread::spawn(move || {
                match stdin.write_all(&input) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Failed to send input into shim instance: {}.", e);
                    }
                };
            });
        }

        let mut stdout = child.stdout.take().unwrap();
        let mut data = vec![];
        let n = stdout
            .read_to_end(&mut data)
            .map_err(|_| FlameError::Internal("failed to read task output".to_string()))?;

        log::debug!("Read <{}> data from child process.", n);

        match child.wait() {
            Ok(es) => {
                if !es.success() {
                    log::info!("Child process exist with error: {}", es);
                }
            }
            Err(e) => {
                log::error!("Failed to wait child process: {}", e)
            }
        };

        Ok(Some(TaskOutput::from(data)))
    }

    async fn on_session_leave(&mut self) -> Result<(), FlameError> {
        Ok(())
    }
}
