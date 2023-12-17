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

// use flame_wasm::*;

// pub struct MatrixServer {}

// impl FlameService for MatrixServer {
//     fn on_session_enter(&self, ctx: SessionContext) -> Result<(), FlameError> {
//         println!("session <{}> enter", ctx.session_id);
//         Ok(())
//     }
//     fn on_session_leave(&self, ctx: SessionContext) -> Result<(), FlameError> {
//         println!("session <{}> leaves", ctx.session_id);
//         Ok(())
//     }

//     fn on_task_invoke(
//         &self,
//         ctx: TaskContext,
//         input: Option<TaskInput>,
//     ) -> Result<Option<TaskOutput>, FlameError> {
//         println!("task <{}/{}> is invoking", ctx.session_id, ctx.task_id);
//         Ok(None)
//     }
// }

fn main() {
    println!("Starting MatrixServer");
    // flame_wasm::run(MatrixServer {})?;

    // Ok(())
}

pub fn on_session_enter() {
    println!("session enter");
}
pub fn on_task_invoke() {
    println!("task invoke");
}
pub fn on_session_leave() {
    println!("session leave");
}
