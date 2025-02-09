/*
Copyright 2025 The Flame Authors.
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

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use common::{lock_ptr, ptr::new_ptr, trace::TraceFn, trace_fn, FlameError};
use rpc::flame::{
    grpc_service_manager_server::{GrpcServiceManager, GrpcServiceManagerServer},
    RegisterServiceRequest, RegisterServiceResponse,
};
use tokio::net::TcpListener;
use tonic::transport::server::TcpIncoming;
use tonic::{transport::Server, Request, Response, Status};

pub type ServiceManagerPtr = Arc<ServiceManager>;

pub async fn new() -> Result<ServiceManagerPtr, FlameError> {
    trace_fn!("ServiceManager::new");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| FlameError::Network(format!("failed to bind tcp listener: {e}")))?;
    let addr = listener
        .local_addr()
        .map_err(|e| FlameError::Network(format!("failed to get local addr: {e}")))?;

    let manager = Arc::new(ServiceManager {
        address: format!("http://127.0.0.1:{}", addr.port()),
        services: new_ptr(HashMap::new()),
    });

    log::debug!("Start service manager at <{addr}>");

    let incoming = TcpIncoming::from_listener(listener, true, Some(Duration::from_secs(1)))
        .map_err(|e| {
            FlameError::Network(format!("failed to create TCP incomming from listener: {e}"))
        })?;
    let manager_backend = manager.clone();
    tokio::spawn(async move {
        Server::builder()
            .add_service(GrpcServiceManagerServer::new(ServiceManagerBackend {
                service_manager: manager_backend,
            }))
            .serve_with_incoming(incoming)
            .await;
    });

    Ok(manager)
}

pub struct ServiceManager {
    address: String,
    services: Arc<Mutex<HashMap<String, String>>>,
}

pub struct ServiceManagerBackend {
    service_manager: ServiceManagerPtr,
}

#[tonic::async_trait]
impl GrpcServiceManager for ServiceManagerBackend {
    async fn register_service(
        &self,
        req: Request<RegisterServiceRequest>,
    ) -> Result<Response<RegisterServiceResponse>, Status> {
        let req = req.into_inner();
        log::debug!(
            "Service <{}> was registered with address <{}>",
            req.service_id,
            req.address
        );

        self.service_manager
            .register_service(req.service_id, req.address)?;

        Ok(Response::new(RegisterServiceResponse {}))
    }
}

impl ServiceManager {
    pub fn get_address(&self) -> String {
        self.address.clone()
    }

    pub fn register_service(&self, sid: String, add: String) -> Result<(), FlameError> {
        let mut services = lock_ptr!(self.services)?;
        services.insert(sid, add);

        Ok(())
    }

    pub async fn get_service(&self, id: &String) -> Result<String, FlameError> {
        WaitForServiceFuture::new(self.services.clone(), id.clone()).await
    }
}

struct WaitForServiceFuture {
    services: Arc<Mutex<HashMap<String, String>>>,
    id: String,
}

impl WaitForServiceFuture {
    pub fn new(services: Arc<Mutex<HashMap<String, String>>>, id: String) -> Self {
        Self { services, id }
    }
}

impl Future for WaitForServiceFuture {
    type Output = Result<String, FlameError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let services = lock_ptr!(self.services)?;
        let svc = services.get(&self.id);

        match svc {
            None => {
                ctx.waker().wake_by_ref();
                Poll::Pending
            }
            Some(svc) => Poll::Ready(Ok(svc.clone())),
        }
    }
}
