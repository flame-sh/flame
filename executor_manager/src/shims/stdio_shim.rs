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

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::executor::{SessionContext, TaskContext};
use crate::shims::Shim;
use common::ptr::MutexPtr;
use common::{lock_ptr, FlameError};

type ChildPtr = MutexPtr<Child>;

#[derive(Clone)]
pub struct StdioShim {
    pub session_context: Option<SessionContext>,
    pub child: Option<ChildPtr>,
}

impl Shim for StdioShim {
    fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        self.session_context = Some(ctx.clone());

        let child = Command::new("/home/klausm/workspace/src/xflops.cn/flame/pi")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| FlameError::Internal("failed to start subprocess".to_string()))?;

        self.child = Some(Arc::new(Mutex::new(child)));

        Ok(())
    }

    fn on_task_invoke(&mut self, ctx: &TaskContext) -> Result<(), FlameError> {
        let mut child = Command::new("/home/klausm/workspace/src/xflops.cn/flame/pi")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| FlameError::Internal("failed to start subprocess".to_string()))?;

        let mut stdin = child.stdin.take().unwrap();
        let input = String::from("10000");
        let handler = thread::spawn(move || {
            match stdin.write_all(input.as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to send input into shim instance: {}.", e);
                }
            };
        });

        let mut reader = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|_| FlameError::Internal("failed to read task output".to_string()))?;

        log::debug!("The output is {}", line);

        handler.join();

        child
            .kill()
            .map_err(|_| FlameError::Internal("failed to kill child process".to_string()))?;

        Ok(())
    }

    fn on_session_leave(&mut self) -> Result<(), FlameError> {
        //        let b = self.child.clone().unwrap();
        //        let mut child = lock_ptr!(b)?;
        //        child
        //            .kill()
        //            .map_err(|_| FlameError::Internal("failed to kill child process".to_string()))?;
        Ok(())
    }
}
