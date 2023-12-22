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

use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::Mutex;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::preview2::{command, Table, WasiCtx, WasiCtxBuilder, WasiView};

use crate::shims::wasm_shim::exports::component::flame::service;
use crate::shims::{Shim, ShimPtr};
use common::{self, apis};

wasmtime::component::bindgen!({
    path: "wit/flame.wit",
    world: "flame",
    async: true
});

pub struct WasmShim {
    session_context: Option<apis::SessionContext>,
    instance: Flame,
    store: Store<ServerWasiView>,
}

impl WasmShim {
    pub async fn new_ptr(app: &apis::Application) -> Result<ShimPtr, common::FlameError> {
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

        let component = Component::from_file(&engine, app.command.clone())
            .context("Component file not found")
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        let (instance, _) = Flame::instantiate_async(&mut store, &component, &linker)
            .await
            .context("Failed to instantiate the flame world")
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        Ok(Arc::new(Mutex::new(WasmShim {
            store,
            instance,
            session_context: None,
        })))
    }
}

#[async_trait]
impl Shim for WasmShim {
    async fn on_session_enter(
        &mut self,
        ctx: &apis::SessionContext,
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
            .map_err(|e| common::FlameError::Internal(e.to_string()))?
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        self.session_context = Some(ctx.clone());

        Ok(())
    }

    async fn on_task_invoke(
        &mut self,
        ctx: &apis::TaskContext,
    ) -> Result<Option<apis::TaskOutput>, common::FlameError> {
        let task_ctx = service::TaskContext {
            session_id: ctx.ssn_id.clone(),
            task_id: ctx.id.clone(),
        };

        let output = self
            .instance
            .interface0
            .call_on_task_invoke(
                &mut self.store,
                &task_ctx,
                ctx.input.clone().map(Bytes::into).as_ref(),
            )
            .await
            .map_err(|e| common::FlameError::Internal(e.to_string()))?
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        Ok(output.map(Bytes::from))
    }

    async fn on_session_leave(&mut self) -> Result<(), common::FlameError> {
        let ssn_ctx = service::SessionContext {
            session_id: self.session_context.clone().unwrap().ssn_id.clone(),
            common_data: None,
        };

        let _ = self
            .instance
            .interface0
            .call_on_session_leave(&mut self.store, &ssn_ctx)
            .await
            .map_err(|e| common::FlameError::Internal(e.to_string()))?
            .map_err(|e| common::FlameError::Internal(e.to_string()))?;

        Ok(())
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
