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

use tokio::runtime::Runtime;
use tonic::transport::Server;

use common::ctx::FlameContext;
use rpc::flame::backend_server::BackendServer;
use rpc::flame::frontend_server::FrontendServer;

use crate::storage::Storage;
use crate::{storage, FlameError, FlameThread};

mod backend;
mod frontend;

pub struct Flame {
    storage: Arc<Storage>,
}

pub fn new() -> Box<dyn FlameThread> {
    Box::new(ApiserverRunner {})
}

struct ApiserverRunner {}

impl FlameThread for ApiserverRunner {
    fn run(&self, ctx: FlameContext) -> Result<(), FlameError> {
        let url = url::Url::parse(&ctx.endpoint)
            .map_err(|_| FlameError::InvalidConfig("invalid endpoint".to_string()))?;
        let host = url
            .host_str()
            .ok_or(FlameError::InvalidConfig("no host in url".to_string()))?;
        let port = url.port().unwrap_or(8080);

        let address_str = format!("{}:{}", host, port);
        log::info!("Listening apiserver at {}", address_str);
        let address = address_str
            .parse()
            .map_err(|_| FlameError::InvalidConfig("failed to parse url".to_string()))?;

        let frontend_service = Flame {
            storage: storage::instance(),
        };

        let backend_service = Flame {
            storage: storage::instance(),
        };

        let rt = Runtime::new()
            .map_err(|_| FlameError::Internal("failed to start tokio runtime".to_string()))?;
        // Execute the future, blocking the current thread until completion
        rt.block_on(async {
            let rc = Server::builder()
                // TODO(k82cn): separate frontend & backend concurrent limit.
                .concurrency_limit_per_connection(6000)
                .add_service(FrontendServer::new(frontend_service))
                .add_service(BackendServer::new(backend_service))
                .serve(address)
                .await;

            match rc {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to run apiserver: {}", e)
                }
            }
        });

        Ok(())
    }
}
