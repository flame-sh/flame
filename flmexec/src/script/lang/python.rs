/*
Copyright 2025 The Flame Authors.
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

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
};

use rand::Rng;

use flame_rs::{apis::FlameError, trace::TraceFn, trace_fn};

use crate::api::{Script, ScriptRuntime};
use crate::script::ScriptEngine;

const DEFAULT_ENTRYPOINT: &str = "main.py";
const UV_CMD: &str = "/usr/bin/uv";

pub struct PythonScript {
    runtime: ScriptRuntime,
}

impl PythonScript {
    pub fn new(script: &Script) -> Result<Self, FlameError> {
        trace_fn!("PythonScript::new");

        let mut rng = rand::rng();
        let work_dir_path = format!("/tmp/flame/script/python-{}", rng.random::<u32>());
        let work_dir = Path::new(&work_dir_path);

        fs::create_dir_all(work_dir).map_err(|e| FlameError::Internal(e.to_string()))?;
        log::debug!("Created work directory: {work_dir_path}");

        let entrypoint = DEFAULT_ENTRYPOINT;

        let mut file = File::create(work_dir.join(entrypoint))
            .map_err(|e| FlameError::Internal(e.to_string()))?;
        file.write_all(script.code.as_bytes())
            .map_err(|e| FlameError::Internal(e.to_string()))?;

        let full_path = work_dir.join(entrypoint);

        let runtime = ScriptRuntime {
            entrypoint: full_path.to_string_lossy().to_string(),
            work_dir: work_dir.to_string_lossy().to_string(),
            input: script.input.clone(),
            env: HashMap::new(),
        };

        Ok(Self { runtime })
    }
}

impl ScriptEngine for PythonScript {
    fn run(&self) -> Result<Option<Vec<u8>>, FlameError> {
        trace_fn!("PythonScript::run");

        log::debug!("Running script: {}", self.runtime.entrypoint);
        log::debug!("Work directory: {}", self.runtime.work_dir);

        let mut child = Command::new(UV_CMD)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(&self.runtime.work_dir)
            .args(["run", &self.runtime.entrypoint])
            .envs(self.runtime.env.iter().map(|(k, v)| (k.clone(), v.clone())))
            .spawn()
            .map_err(|e| FlameError::Internal(format!("failed to start subprocess: {e}")))?;

        log::debug!("Spawned child process: {}", child.id());
        let mut stdin = child.stdin.take().unwrap();
        if let Some(input) = &self.runtime.input {
            let input = input.clone();
            let _handler = thread::spawn(move || {
                match stdin.write_all(&input) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Failed to send input into shim instance: {e}.");
                    }
                };
            });
            log::debug!("Sent input into child process.");
        }

        let mut stdout = child.stdout.take().unwrap();
        let mut data = vec![];
        let n = stdout
            .read_to_end(&mut data)
            .map_err(|_| FlameError::Internal("failed to read task output".to_string()))?;

        log::debug!("Read <{n}> data from child process.");

        match child.wait() {
            Ok(es) => {
                if !es.success() {
                    log::info!("Child process exist with error: {es}");
                }
            }
            Err(e) => {
                log::error!("Failed to wait child process: {e}")
            }
        };

        log::debug!("Child process exited.");

        Ok(Some(data))
    }
}

impl Drop for PythonScript {
    fn drop(&mut self) {
        trace_fn!("PythonScript::drop");

        fs::remove_dir_all(Path::new(&self.runtime.work_dir)).unwrap();
    }
}
