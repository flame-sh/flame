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

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use tokio::sync::Mutex;

use crate::shims::{Shim, ShimPtr};
use common::apis::{Application, SessionContext, TaskContext, TaskOutput};
use common::FlameError;

#[derive(Clone)]
pub struct PythonShim {
    task_code: Option<Bytes>,
}

impl PythonShim {
    pub fn new_ptr(_: &Application) -> ShimPtr {
        Arc::new(Mutex::new(Self { task_code: None }))
    }
}

#[async_trait]
impl Shim for PythonShim {
    async fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        self.task_code = ctx.common_data.clone();

        Ok(())
    }

    async fn on_task_invoke(
        &mut self,
        ctx: &TaskContext,
    ) -> Result<Option<TaskOutput>, FlameError> {
        Python::with_gil(|py| {
            let dill = py
                .import("dill")
                .map_err(|e| FlameError::Internal(e.to_string()))?;
            let load_fn = dill
                .getattr("loads")
                .map_err(|e| FlameError::Internal(e.to_string()))?;

            let dump_fn = dill
                .getattr("dumps")
                .map_err(|e| FlameError::Internal(e.to_string()))?;

            // Load task code
            let task_code = PyBytes::new(py, &self.task_code.clone().unwrap());
            let task_code = PyTuple::new(py, &[task_code]);
            let task_fn: Py<PyAny> = load_fn
                .call(task_code, None)
                .map_err(|e| FlameError::Internal(e.to_string()))?
                .into();

            // Load args of task
            let task_args = PyBytes::new(py, &ctx.input.clone().unwrap());
            let task_args = PyTuple::new(py, &[task_args]);
            let args = load_fn
                .call(task_args, None)
                .map_err(|e| FlameError::Internal(e.to_string()))?;
            let args: &PyTuple = args
                .extract()
                .map_err(|e| FlameError::Internal(e.to_string()))?;

            // Execute the python task
            let res = task_fn
                .call(py, args, None)
                .map_err(|e| FlameError::Internal(e.to_string()))?;

            // Dump the output into Bytes
            let res = PyTuple::new(py, &[res]);
            let any: &PyAny = dump_fn
                .call(res, None)
                .map_err(|e| FlameError::Internal(e.to_string()))?;

            let py_bytes: &PyBytes = any
                .downcast()
                .map_err(|e| FlameError::Internal(e.to_string()))?;
            let output: Bytes = Bytes::copy_from_slice(py_bytes.as_bytes());

            // Return output
            Ok(Some(output))
        })
    }

    async fn on_session_leave(&mut self) -> Result<(), FlameError> {
        Ok(())
    }
}
