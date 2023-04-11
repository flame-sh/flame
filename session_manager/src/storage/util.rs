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
use std::ops::Deref;
use crate::FlameError;

pub(crate) fn next_id(id: &Mutex<i64>) -> Result<i64, FlameError> {
    let mut id = id.lock().map_err(|_| {
        FlameError::Mutex("max id".to_string())
    })?;
    *id = *id + 1;

    Ok(*id.deref())
}