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

use crate::scheduler::ctx::Context;

use crate::storage::StoragePtr;
use crate::FlameThread;
use common::ctx::FlameContext;
use common::FlameError;

mod actions;
mod ctx;
mod plugins;

pub fn new(storage: StoragePtr) -> Box<dyn FlameThread> {
    Box::new(ScheduleRunner { storage })
}

struct ScheduleRunner {
    storage: StoragePtr,
}

impl FlameThread for ScheduleRunner {
    fn run(&self, _flame_ctx: FlameContext) -> Result<(), FlameError> {
        loop {
            let mut ctx = Context::new(self.storage.clone())?;

            for action in ctx.actions.clone() {
                if let Err(e) = action.execute(&mut ctx) {
                    log::error!("Failed to run scheduling: {}", e);
                    break;
                };
            }

            let delay = time::Duration::from_millis(ctx.schedule_interval);
            thread::sleep(delay);
        }
    }
}
