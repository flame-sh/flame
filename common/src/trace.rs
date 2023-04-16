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

pub struct TraceFn{
    pub fn_name : String,
}

impl TraceFn {
    pub fn new(n: String) -> Self {
        log::debug!("{} Enter", n);
        TraceFn {fn_name: n}
    }
}

impl Drop for TraceFn {
    fn drop(&mut self) {
        log::debug!("{} Leaving", self.fn_name);
    }
}

#[macro_export]
macro_rules! trace_fn {
    ($e:expr) => (
        let _trace_fn = TraceFn::new($e.to_string());
        // let _scope_call = TraceFn { fn_name: $e.to_string() };
    )
}