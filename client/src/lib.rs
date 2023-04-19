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

pub fn connect() {}

pub fn create_session() {}

pub struct Session {}

pub struct Task {}

pub struct TaskInput {}
pub struct TaskOutput {}

type TaskID = String;
// type SessionID = String;

impl Session {
    pub fn new() -> Self {
        todo!()
    }

    pub fn create_task(&self, _: &TaskInput) -> Task {
        todo!()
    }

    pub fn get_task(&self, _: TaskID) -> Task {
        todo!()
    }

    pub fn watch_task(&self, _: TaskID) {
        todo!()
    }

    // pub fn get_task_output(&self, _: TaskID) -> TaskOutput {
    //     todo!()
    // }
}

impl Drop for Session {
    fn drop(&mut self) {
        todo!()
    }
}
