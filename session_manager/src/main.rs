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

use common::{FlameContext, FlameError};
use std::thread;

mod apiserver;
mod model;
mod scheduler;
mod storage;

#[tokio::main]
async fn main() -> Result<(), FlameError> {
    env_logger::init();

    let ctx = FlameContext::from_file(None)?;

    log::info!("flame-session-manager is starting ...");

    // storage::start()?;
    // scheduler::start()?;
    // log::debug!("scheduler was started.");

    // apiserver::run().await?;

    let mut handlers = vec![];
    let threads = vec![scheduler::new(), apiserver::new()];

    for thread in threads {
        let ctx = ctx.clone();
        let handler = thread::spawn(move || {
            match thread.run(ctx) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to run thread: {}", e);
                }
            };
        });

        handlers.push(handler);
    }

    log::info!("flame-session-manager started.");

    for h in handlers {
        h.join().unwrap();
    }

    Ok(())
}

pub trait FlameThread: Send + Sync + 'static {
    fn run(&self, ctx: FlameContext) -> Result<(), FlameError>;
}
//
// struct ThreadManager {
//     pub threads: HashMap<String, Box<dyn FlameThread>>,
// }
//
// impl ThreadManager {
//     pub fn run(&self) -> Result<(), FlameError> {
//         let mut handlers = HashMap::new();
//
//         let ctx = FlameContext::from_file(None)?;
//
//         for (n, t) in self.threads.iter().clone() {
//             let ctx = ctx.clone();
//             let handler = thread::spawn(move||{
//                 match t.run(ctx){
//                     Ok(_) => {}
//                     Err(e) => {
//                         log::error!("Failed to run thread: {}", e);
//                     }
//                 };
//             });
//
//             handlers.insert(n, handler);
//         }
//
//
//
//         Ok(())
//     }
//
// }
