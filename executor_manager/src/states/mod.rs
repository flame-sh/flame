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

use rpc::flame::frontend_client::FrontendClient;

use crate::executor::{Executor, ExecutorState};
use common::{FlameContext, FlameError};

mod bound;
mod idle;
mod init;
mod running;
mod unknown;

pub fn get_state(e: &Executor) -> Result<Box<dyn State>, FlameError> {
    match e.state {
        ExecutorState::Initialized => Ok(Box::new(int::InitState {})),
        ExecutorState::Idle => {}
        ExecutorState::Bound => {}
        ExecutorState::Running => {}
        ExecutorState::Unknown => {}
    }
}

pub trait State {
    fn execute<T>(
        &self,
        ctx: &FlameContext,
        client: &mut FrontendClient<T>,
    ) -> Result<(), FlameError>;
}
