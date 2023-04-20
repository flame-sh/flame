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

use std::collections::HashMap;
use std::thread;

use clap::Parser;

use common::ctx::FlameContext;
use common::FlameError;

mod apiserver;
mod model;
mod scheduler;
mod storage;

#[derive(Parser)]
#[command(name = "flame-session-manager")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Session Manager", long_about = None)]
struct Cli {
    #[arg(long)]
    flame_conf: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), FlameError> {
    env_logger::init();

    let cli = Cli::parse();
    let ctx = FlameContext::from_file(cli.flame_conf)?;

    log::info!("flame-session-manager is starting ...");

    let mut handlers = vec![];
    let mut threads = HashMap::new();
    threads.insert("scheduler", scheduler::new());
    threads.insert("apiserver", apiserver::new());

    for (n, thread) in threads {
        let ctx = ctx.clone();
        let handler = thread::spawn(move || {
            match thread.run(ctx) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to run thread: {}", e);
                }
            };
        });

        log::info!("<{}> thread was started.", n);

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
