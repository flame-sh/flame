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

use clap::Parser;
use futures::future::join_all;
use std::io::Write;
use std::process;

use chrono::{Duration, Utc};
use std::collections::HashMap;

use common::apis::{
    ApplicationAttributes, ApplicationState, Shim, DEFAULT_DELAY_RELEASE, DEFAULT_MAX_INSTANCES,
};
use common::ctx::FlameContext;
use common::FlameError;

mod apiserver;
mod controller;
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
    common::init_logger();

    let cli = Cli::parse();
    let ctx = FlameContext::from_file(cli.flame_conf)?;

    log::info!("flame-session-manager is starting ...");

    let mut handlers = vec![];

    let storage = storage::new_ptr(&ctx.storage).await?;

    // Load data from engine, e.g. sqlite.
    storage.load_data().await?;

    let controller = controller::new_ptr(storage.clone());

    // Start apiserver thread.
    {
        let controller = controller.clone();
        let ctx = ctx.clone();
        let handler = tokio::spawn(async move {
            let apiserver = apiserver::new(controller);
            apiserver.run(ctx).await
        });
        handlers.push(handler);
    }

    // Start scheduler thread.
    {
        let controller = controller.clone();
        let ctx = ctx.clone();
        let handler = tokio::spawn(async move {
            let scheduler = scheduler::new(controller);
            scheduler.run(ctx).await
        });
        handlers.push(handler);
    }

    log::info!("flame-session-manager started.");

    // Register default applications.
    for (name, attr) in common::default_applications() {
        controller.register_application(name, attr).await?;
    }

    // Waiting for all thread to exit.
    let _ = join_all(handlers).await;

    Ok(())
}

#[async_trait::async_trait]
pub trait FlameThread: Send + Sync + 'static {
    async fn run(&self, ctx: FlameContext) -> Result<(), FlameError>;
}
