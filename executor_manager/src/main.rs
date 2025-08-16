/*
Copyright 2023 The Flame Authors.
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

use crate::manager::ExecutorManager;
use clap::Parser;
use common::ctx::FlameContext;
use common::FlameError;

mod client;
mod executor;
mod manager;
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
async fn main() -> Result<(), FlameError> {
    common::init_logger();

    let cli = Cli::parse();
    let ctx = FlameContext::from_file(cli.flame_conf)?;

    // Create the executor manager by the context.
    let mut manager = ExecutorManager::new(&ctx).await?;

    // Run the executor manager.
    manager.run().await?;

    Ok(())
}
