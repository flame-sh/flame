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

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

cargo_component_bindings::generate!();

use crate::bindings::exports::component::flame::service::{
    FlameError, FlameErrorCode, Guest, SessionContext, TaskContext, TaskInput, TaskOutput,
};

#[derive(Default, Serialize, Deserialize)]
struct MatrixData {
    n: u16,
    u: Vec<Vec<i32>>,
    v: Vec<Vec<i32>>,
}

static DATA: Mutex<MatrixData> = Mutex::new(MatrixData {
    n: 0,
    u: vec![],
    v: vec![],
});

struct Component {}

impl Guest for Component {
    fn on_session_enter(ctx: SessionContext) -> Result<(), FlameError> {
        match ctx.common_data {
            None => Err(FlameError {
                code: FlameErrorCode::Internal,
                message: String::from("common data is empty"),
            }),
            Some(data) => {
                let mut dataptr = DATA.lock().map_err(|e| FlameError {
                    code: FlameErrorCode::Internal,
                    message: e.to_string(),
                })?;
                *dataptr = serde_json::from_slice(&data).map_err(|e| FlameError {
                    code: FlameErrorCode::Internal,
                    message: e.to_string(),
                })?;

                Ok(())
            }
        }
    }

    fn on_session_leave(_: SessionContext) -> Result<(), FlameError> {
        Ok(())
    }

    fn on_task_invoke(
        _: TaskContext,
        input: Option<TaskInput>,
    ) -> Result<Option<TaskOutput>, FlameError> {
        match input {
            None => Err(FlameError {
                code: FlameErrorCode::Internal,
                message: String::from("task input is empty"),
            }),
            Some(input) => {
                let dataptr = DATA.lock().map_err(|e| FlameError {
                    code: FlameErrorCode::Internal,
                    message: e.to_string(),
                })?;

                let task_id = u16::from_ne_bytes(input.try_into().map_err(|_| FlameError {
                    code: FlameErrorCode::Internal,
                    message: String::from("failed to parse task input"),
                })?);

                let (m, n) = (
                    (task_id / dataptr.n) as usize,
                    (task_id % dataptr.n) as usize,
                );

                let u = &dataptr.u[m];

                let v = {
                    let mut v = vec![];
                    for i in 0..dataptr.n {
                        v.push(dataptr.v[i as usize][n])
                    }

                    v
                };

                let mut s: i32 = 0;
                for i in 0..dataptr.n {
                    s += v[i as usize] * u[i as usize];
                }

                Ok(Some(s.to_ne_bytes().to_vec()))
            }
        }
    }
}
