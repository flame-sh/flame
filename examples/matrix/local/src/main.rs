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
use clap::Parser;

use anyhow::Context;
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::preview2::{command, Table, WasiCtx, WasiCtxBuilder, WasiView};

use crate::exports::component::flame::service::{SessionContext, TaskContext};

wasmtime::component::bindgen!({
    path: "wit/flame.wit",
    world: "flame",
    async: true
});

#[derive(Error, Debug)]
pub enum LocalError {
    #[error("{0}")]
    Internal(String),
}

#[derive(Default, Serialize, Deserialize)]
struct MatrixData {
    n: u16,
    u: Vec<Vec<i32>>,
    v: Vec<Vec<i32>>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    command: String,

    /// Number of times to greet
    #[arg(short = 'n', long, default_value_t = 3)]
    size: u16,

    #[arg(short, long, default_value = "1")]
    session: String,
}

#[tokio::main]
async fn main() -> Result<(), LocalError> {
    let args = Args::parse();

    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(true);

    let engine = Engine::new(&config).map_err(|e| LocalError::Internal(e.to_string()))?;
    let mut linker = Linker::new(&engine);
    command::add_to_linker(&mut linker).map_err(|e| LocalError::Internal(e.to_string()))?;
    let wasi_view = ServerWasiView::new();
    let mut store = Store::new(&engine, wasi_view);

    let component = Component::from_file(&engine, args.command)
        .context("Component file not found")
        .map_err(|e| LocalError::Internal(e.to_string()))?;

    let (instance, _) = Flame::instantiate_async(&mut store, &component, &linker)
        .await
        .context("Failed to instantiate the flame world")
        .map_err(|e| LocalError::Internal(e.to_string()))?;

    let mut data = MatrixData {
        n: args.size,
        ..MatrixData::default()
    };

    let mut rng = rand::thread_rng();

    for _ in 0..data.n {
        let mut u = vec![];
        for _ in 0..data.n {
            u.push(rng.gen_range(0..10));
        }
        data.u.push(u);
    }

    for _ in 0..data.n {
        let mut v = vec![];
        for _ in 0..data.n {
            v.push(rng.gen_range(0..10));
        }
        data.v.push(v);
    }

    let common_data =
        serde_json::to_string(&data).map_err(|e| LocalError::Internal(e.to_string()))?;

    println!("{}\n", common_data);

    let ssn_ctx = SessionContext {
        session_id: args.session.clone(),
        common_data: Some(common_data.into()),
    };

    instance
        .interface0
        .call_on_session_enter(&mut store, &ssn_ctx)
        .await
        .map_err(|e| LocalError::Internal(e.to_string()))?
        .map_err(|e| LocalError::Internal(e.to_string()))?;

    for i in 0..data.n * data.n {
        let task_ctx = TaskContext {
            session_id: args.session.clone(),
            task_id: i.to_string(),
        };

        let input = i.to_ne_bytes().to_vec();

        let output = instance
            .interface0
            .call_on_task_invoke(&mut store, &task_ctx, Some(&input))
            .await
            .map_err(|e| LocalError::Internal(e.to_string()))?
            .map_err(|e| LocalError::Internal(e.to_string()))?;

        if let Some(o) = output {
            print!(
                "{} ",
                i32::from_ne_bytes(o.try_into().map_err(|_| LocalError::Internal(
                    String::from("failed to parse task output")
                ))?)
            );
        }

        if i % data.n == data.n - 1 {
            println!();
        }
    }

    instance
        .interface0
        .call_on_session_leave(&mut store, &ssn_ctx)
        .await
        .map_err(|e| LocalError::Internal(e.to_string()))?
        .map_err(|e| LocalError::Internal(e.to_string()))?;

    Ok(())
}

struct ServerWasiView {
    table: Table,
    ctx: WasiCtx,
}

impl ServerWasiView {
    fn new() -> Self {
        let table = Table::new();
        let ctx = WasiCtxBuilder::new().inherit_stdio().build();

        Self { table, ctx }
    }
}

impl WasiView for ServerWasiView {
    fn table(&self) -> &Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.ctx
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
