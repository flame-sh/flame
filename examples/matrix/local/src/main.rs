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

use common::{self, apis};

use anyhow::Context;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::preview2::{command, Table, WasiCtx, WasiCtxBuilder, WasiView};

use crate::exports::component::matrix::service;

wasmtime::component::bindgen!({
    path: "flame.wit",
    world: "flame",
    async: true
});

pub struct WasmShim {
    instance: Flame,
    store: Store<ServerWasiView>,
}

impl WasmShim {
    pub async fn new() -> Result<Self, common::FlameError> {
        let mut config = Config::default();

        config.wasm_component_model(true);
        config.async_support(true);
        let engine =
            Engine::new(&config).map_err(|e| common::FlameError::Internal(e.to_string()))?;
        let mut linker = Linker::new(&engine);

        command::add_to_linker(&mut linker)
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;
        let wasi_view = ServerWasiView::new();
        let mut store = Store::new(&engine, wasi_view);

        let component =
            Component::from_file(&engine, "target/wasm32-wasi/debug/matrix_server.wasm")
                .context("Component file not found")
                .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        let (instance, _) = Flame::instantiate_async(&mut store, &component, &linker)
            .await
            .context("Failed to instantiate the flame world")
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        Ok(WasmShim { store, instance })
    }

    pub async fn on_session_enter(
        &mut self,
        ctx: apis::SessionContext,
    ) -> Result<(), common::FlameError> {
        let ssn_ctx = service::SessionContext {
            session_id: ctx.ssn_id.clone(),
            common_data: None,
        };

        let _ = self
            .instance
            .interface0
            .call_on_session_enter(&mut self.store, &ssn_ctx)
            .await
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn on_session_leave(
        &mut self,
        ctx: apis::SessionContext,
    ) -> Result<(), common::FlameError> {
        let ssn_ctx = service::SessionContext {
            session_id: ctx.ssn_id.clone(),
            common_data: None,
        };

        let _ = self
            .instance
            .interface0
            .call_on_session_leave(&mut self.store, &ssn_ctx)
            .await
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn on_task_invoke(
        &mut self,
        ctx: apis::TaskContext,
    ) -> Result<Option<apis::TaskOutput>, common::FlameError> {
        let task_ctx = service::TaskContext {
            session_id: ctx.ssn_id.clone(),
            task_id: ctx.id.clone(),
        };

        let _ = self
            .instance
            .interface0
            .call_on_task_invoke(&mut self.store, &task_ctx, None)
            .await
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        Ok(None)
    }
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

#[tokio::main]
async fn main() -> Result<(), common::FlameError> {
    let mut shim = WasmShim::new().await?;

    let ctx = apis::SessionContext {
        ssn_id: "1".to_string(),
        slots: 1,
        application: "wasm".to_string(),
    };

    shim.on_session_enter(ctx.clone()).await?;

    for i in 0..3 {
        shim.on_task_invoke(apis::TaskContext {
            id: i.to_string(),
            ssn_id: ctx.ssn_id.clone(),
            input: None,
            output: None,
        })
        .await?;
    }

    shim.on_session_leave(ctx.clone()).await?;

    Ok(())
}
