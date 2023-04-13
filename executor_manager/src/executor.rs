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

use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug)]
pub enum ExecutorState {
    Initialized = 0,
    Idle = 1,
    Bound = 2,
    Running = 3,
    Unknown = 4,
}

#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environments: Vec<String>,
    pub working_directory: String,
}

#[derive(Clone, Debug)]
pub struct Task {
    id: String,
    ssn_id: String,
    input: String,
}

#[derive(Clone, Debug)]
pub struct Executor {
    pub id: String,
    pub slots: i32,
    pub application: Application,
    pub task: Option<Task>,

    pub start_time: DateTime<Utc>,
    pub state: ExecutorState,
}
