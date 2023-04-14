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
use tonic::transport::Server;

use rpc::flame::backend_server::BackendServer;
use rpc::flame::frontend_server::FrontendServer;

use crate::storage::Storage;
use crate::{storage, FlameError};

mod backend;
mod frontend;

pub struct Flame {
    storage: Arc<Storage>,
}

pub async fn run() -> Result<(), FlameError> {
    let address = "[::1]:8080".parse().unwrap();
    let frontend_service = Flame {
        storage: storage::instance(),
    };

    let backend_service = Flame {
        storage: storage::instance(),
    };

    Server::builder()
        .add_service(FrontendServer::new(frontend_service))
        .add_service(BackendServer::new(backend_service))
        .serve(address)
        .await
        .map_err(|e| FlameError::Network(e.to_string()))?;

    Ok(())
}
