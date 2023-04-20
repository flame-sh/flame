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

use std::{thread, time};

use crate::scheduler::actions::{Action, AllocateAction};
use crate::{storage, FlameThread};
use common::ctx::FlameContext;
use common::FlameError;

mod actions;

pub fn new() -> Box<dyn FlameThread> {
    Box::new(SchedulerRunner {})
}

struct SchedulerRunner {}

impl FlameThread for SchedulerRunner {
    fn run(&self, _ctx: FlameContext) -> Result<(), FlameError> {
        loop {
            let mut snapshot = storage::instance().snapshot()?;
            let actions: Vec<Box<dyn Action>> = vec![Box::new(AllocateAction {
                storage: storage::instance(),
            })];

            for action in actions {
                if let Err(e) = action.execute(&mut snapshot) {
                    log::error!("Failed to run scheduling: {}", e);
                    break;
                };
            }

            let delay = time::Duration::from_millis(500);
            thread::sleep(delay);
        }
    }
}
