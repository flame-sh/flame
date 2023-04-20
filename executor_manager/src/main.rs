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

use std::error::Error;

use crate::executor::Executor;
use clap::Parser;
use common::ctx::FlameContext;

mod client;
mod executor;
mod shims;
mod states;

#[derive(Parser)]
#[command(name = "flame-executor-manager")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Executor Manager", long_about = None)]
struct Cli {
    #[arg(long)]
    flame_conf: Option<String>,
    #[arg(long)]
    slots: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();
    let ctx = FlameContext::from_file(cli.flame_conf)?;

    // Setup Flame backend client.
    client::install(&ctx).await?;

    // Run executor.
    // TODO(k82cn): 1. enable gracefully exit, 2. build ExecutorManager for multiple executors.
    let mut exec = Executor::from_context(&ctx, cli.slots).await?;
    // let mut exec_ptr = ExecutorPtr::new(exec);

    loop {
        let mut state = states::from(exec.clone());
        match state.execute(&ctx).await {
            Ok(next_state) => {
                exec.update_state(&next_state);
            }
            Err(e) => {
                log::error!("Failed to execute: {}", e);
            }
        }
    }
}
