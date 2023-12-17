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

use common::{apis::*, *};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

pub struct WasmShim {
    instance: Instance,
    store: Store<WasiCtx>,
}

impl WasmShim {
    pub fn new() -> Result<Self, FlameError> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)
            .map_err(|e| FlameError::Internal(e.to_string()))?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()
            .expect("should always be able to inherit args")
            .build();
        let mut store = Store::new(&engine, wasi);
        let module = Module::from_file(&engine, "target/wasm32-wasi/debug/matrix_server.wasm")
            .map_err(|e| FlameError::Internal(e.to_string()))?;

        // let instance = Instance::new(&mut store, &module, &[])
        //     .map_err(|e| FlameError::Internal(e.to_string()))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| FlameError::NotFound(e.to_string()))?;

        Ok(WasmShim { store, instance })
    }

    pub async fn on_session_enter(&mut self, ctx: SessionContext) -> Result<(), FlameError> {
        let ssn_enter = self
            .instance
            .get_func(&mut self.store, "on_session_enter")
            .expect("`on_session_enter` was not an exported function");

        ssn_enter
            .call(&mut self.store, &[], &mut [])
            .map_err(|e| FlameError::NotFound(e.to_string()))?;

        Ok(())
    }

    pub async fn on_session_leave(&mut self, ctx: SessionContext) -> Result<(), FlameError> {
        let ssn_leave = self
            .instance
            .get_func(&mut self.store, "on_session_leave")
            .expect("`on_session_leave` was not an exported function");

        ssn_leave
            .call(&mut self.store, &[], &mut [])
            .map_err(|e| FlameError::NotFound(e.to_string()))?;

        Ok(())
    }

    pub async fn on_task_invoke(&mut self, ctx: TaskContext) -> Result<Option<TaskOutput>, FlameError> {
        let task_invoke = self
            .instance
            .get_func(&mut self.store, "on_task_invoke")
            .expect("`on_session_leave` was not an exported function");

        task_invoke
            .call(&mut self.store, &[], &mut [])
            .map_err(|e| FlameError::NotFound(e.to_string()))?;

        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<(), FlameError> {
    let mut shim = WasmShim::new()?;

    let ctx = SessionContext {
        ssn_id: "1".to_string(),
        slots: 1,
        application: "wasm".to_string(),
    };

    shim.on_session_enter(ctx.clone()).await?;

    for i in 0..3 {
        shim.on_task_invoke(TaskContext {
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
