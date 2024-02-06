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

use std::error::Error;

use clap::{Parser, Subcommand};
use common::ctx::FlameContext;

mod create;
mod helper;
mod list;
mod migrate;
mod view;

#[derive(Parser)]
#[command(name = "flmctl")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame command line", long_about = None)]
struct Cli {
    #[arg(long)]
    flame_conf: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    View {
        #[arg(short, long)]
        session: String,
    },
    List,
    Close {
        #[arg(short, long)]
        session: String,
    },
    Create {
        #[arg(short, long)]
        app: String,
        #[arg(short, long)]
        slots: i32,
    },
    Migrate {
        #[arg(short, long)]
        url: String,
        #[arg(short, long)]
        sql: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cli = Cli::parse();
    let ctx = FlameContext::from_file(cli.flame_conf)?;

    match &cli.command {
        Some(Commands::List) => list::run(&ctx).await?,
        Some(Commands::Close { .. }) => {
            todo!()
        }
        Some(Commands::Create { app, slots }) => create::run(&ctx, app, slots).await?,
        Some(Commands::View { session }) => view::run(&ctx, session).await?,
        Some(Commands::Migrate { url, sql }) => migrate::run(&ctx, url, sql).await?,
        _ => helper::run().await?,
    };

    Ok(())
}
