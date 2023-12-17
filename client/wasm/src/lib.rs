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

// use std::sync::mpsc;
// use std::sync::mpsc::{Receiver, Sender};
// use std::sync::Mutex;
// use thiserror::Error;

// use bytes::Bytes;
// use lazy_static::lazy_static;

// #[derive(Error, Debug, Clone)]
// pub enum FlameError {
//     #[error("'{0}' not found")]
//     NotFound(String),

//     #[error("'{0}'")]
//     Internal(String),

//     #[error("'{0}'")]
//     Network(String),

//     #[error("'{0}'")]
//     InvalidConfig(String),
// }

// #[macro_export]
// macro_rules! lock_ptr {
//     ( $mutex_arc:expr ) => {
//         $mutex_arc
//             .lock()
//             .map_err(|_| FlameError::Internal("mutex ptr".to_string()))
//     };
// }

// pub struct SessionContext {
//     pub session_id: String,
//     pub common_data: Option<Bytes>,
// }

// pub type TaskInput = bytes::Bytes;
// pub type TaskOutput = bytes::Bytes;

// pub struct TaskContext {
//     pub session_id: String,
//     pub task_id: String,
// }

// pub trait FlameService {
//     fn on_session_enter(&self, ctx: SessionContext) -> Result<(), FlameError>;
//     fn on_session_leave(&self, ctx: SessionContext) -> Result<(), FlameError>;

//     fn on_task_invoke(
//         &self,
//         ctx: TaskContext,
//         input: Option<TaskInput>,
//     ) -> Result<Option<TaskOutput>, FlameError>;
// }

// pub fn run<S: FlameService>(svc: S) -> Result<(), FlameError> {
//     loop {
//         let msg = lock_ptr!(FLM_TO_WASM)?.1.recv().unwrap();
//         let res = match msg.cmd {
//             Command::EnterSession => {
//                 let ctx = SessionContext {
//                     session_id: msg.session_id.unwrap(),
//                     common_data: msg.common_data,
//                 };
//                 svc.on_session_enter(ctx).map(|_| None)
//             }
//             Command::LeaveSession => {
//                 let ctx = SessionContext {
//                     session_id: msg.session_id.unwrap(),
//                     common_data: msg.common_data,
//                 };
//                 svc.on_session_leave(ctx).map(|_| None)
//             }
//             Command::InvokeTask => {
//                 let ctx = TaskContext {
//                     session_id: msg.session_id.unwrap(),
//                     task_id: msg.task_id.unwrap(),
//                 };
//                 svc.on_task_invoke(ctx, msg.task_input)
//             }
//         };

//         lock_ptr!(WASM_TO_FLM)?.0.send(res).unwrap();

//         if msg.cmd == Command::LeaveSession {
//             break;
//         }
//     }
//     Ok(())
// }

// #[derive(PartialEq)]
// enum Command {
//     EnterSession,
//     LeaveSession,
//     InvokeTask,
// }
// struct Message {
//     pub cmd: Command,
//     session_id: Option<String>,
//     common_data: Option<Bytes>,
//     task_id: Option<String>,
//     task_input: Option<TaskInput>,
//     task_output: Option<TaskOutput>,
// }

// lazy_static! {
//     static ref FLM_TO_WASM: Mutex<(Sender<Message>, Receiver<Message>)> =
//         Mutex::new(mpsc::channel());
//     static ref WASM_TO_FLM: Mutex<(
//         Sender<Result<Option<TaskOutput>, FlameError>>,
//         Receiver<Result<Option<TaskOutput>, FlameError>>
//     )> = Mutex::new(mpsc::channel());
// }

pub fn on_session_enter() {
    println!("session enter");
}
pub fn on_task_invoke() {
    println!("task invoke");
}
pub fn on_session_leave() {
    println!("session leave");
}
