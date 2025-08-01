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

use std::env;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;

use common::ctx::FlameContext;
use rpc::flame::backend_server::BackendServer;
use rpc::flame::frontend_server::FrontendServer;

use crate::controller::ControllerPtr;
use crate::{FlameError, FlameThread};

mod backend;
mod frontend;

pub struct Flame {
    controller: ControllerPtr,
}

pub fn new(controller: ControllerPtr) -> Arc<dyn FlameThread> {
    Arc::new(ApiserverRunner { controller })
}

struct ApiserverRunner {
    controller: ControllerPtr,
}

const DEFAULT_PORT: u16 = 8080;
const ALL_HOST_ADDRESS: &str = "0.0.0.0";

#[async_trait::async_trait]
impl FlameThread for ApiserverRunner {
    async fn run(&self, ctx: FlameContext) -> Result<(), FlameError> {
        let url = url::Url::parse(&ctx.endpoint)
            .map_err(|_| FlameError::InvalidConfig("invalid endpoint".to_string()))?;
        let port = url.port().unwrap_or(DEFAULT_PORT);

        // The fsm will bind to all addresses of host directly.
        let address_str = format!("{}:{}", ALL_HOST_ADDRESS, port);
        log::info!("Listening apiserver at {}", address_str);
        let address = address_str
            .parse()
            .map_err(|_| FlameError::InvalidConfig("failed to parse url".to_string()))?;

        let frontend_service = Flame {
            controller: self.controller.clone(),
        };

        let backend_service = Flame {
            controller: self.controller.clone(),
        };

        Server::builder()
            .tcp_keepalive(Some(Duration::from_secs(1)))
            .add_service(FrontendServer::new(frontend_service))
            .add_service(BackendServer::new(backend_service))
            .serve(address)
            .await
            .map_err(|e| FlameError::Network(e.to_string()))?;

        Ok(())
    }
}
