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

use std::env::{self, VarError};

use gethostname::gethostname;

const FLAME_TASK_ID: &str = "FLAME_TASK_ID";
const FLAME_SESSION_ID: &str = "FLAME_SESSION_ID";

pub fn main() -> Result<(), VarError> {
    let ssn_id = env::var(FLAME_SESSION_ID)?;
    let task_id = env::var(FLAME_TASK_ID)?;

    print!("Execute <{ssn_id}/{task_id}> on host <{:?}>", gethostname());

    Ok(())
}
