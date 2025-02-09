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

use std::env;
use std::fs::File;
use std::process::{self, Stdio};
use std::sync::Arc;
use std::time::Duration;
use std::{thread, time};

use async_trait::async_trait;
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use tonic::transport::{Endpoint, Uri};
use tonic::Request;
use tower::service_fn;

use ::rpc::flame as rpc;
use rpc::grpc_shim_client::GrpcShimClient;
use rpc::EmptyRequest;
use uuid::Uuid;

use crate::shims::{Shim, ShimPtr};
use crate::svcmgr::ServiceManagerPtr;
use common::apis::{ApplicationContext, SessionContext, TaskContext, TaskOutput};
use common::{trace::TraceFn, trace_fn, FlameError};

pub struct GrpcShim {
    session_context: Option<SessionContext>,
    client: GrpcShimClient<Channel>,
    child: tokio::process::Child,
}

const FLAME_SERVICE_MANAGER: &str = "FLAME_SERVICE_MANAGER";
const RUST_LOG: &str = "RUST_LOG";
const DEFAULT_SVC_LOG_LEVEL: &str = "info";

impl GrpcShim {
    pub async fn new_ptr(
        app: &ApplicationContext,
        servce_manager: ServiceManagerPtr,
    ) -> Result<ShimPtr, FlameError> {
        // Spawn child process
        let mut cmd = tokio::process::Command::new(&app.command.clone().unwrap());
        let log_level = env::var(RUST_LOG).unwrap_or(String::from(DEFAULT_SVC_LOG_LEVEL));

        let mut child = cmd
            .env(FLAME_SERVICE_MANAGER, &servce_manager.get_address())
            .env(RUST_LOG, log_level)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                FlameError::InvalidConfig(format!(
                    "failed to start service by command <{}>: {e}",
                    app.command.clone().unwrap_or_default()
                ))
            })?;

        let service_id = child.id().unwrap_or_default();

        log::debug!(
            "The service <{}> was started, waiting for registering.",
            service_id
        );

        let addr = servce_manager.get_service(&service_id.to_string()).await?;
        log::debug!("Try to connect to service <{}> at <{}>", service_id, addr);

        let client = GrpcShimClient::connect(addr).await.map_err(|e| {
            FlameError::Network(format!("failed to connect to service <{service_id}>: {e}"))
        })?;

        Ok(Arc::new(Mutex::new(Self {
            session_context: None,
            client,
            child,
        })))
    }
}

#[async_trait]
impl Shim for GrpcShim {
    async fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        trace_fn!("GrpcShim::on_session_enter");

        let req = Request::new(rpc::SessionContext::from(ctx.clone()));
        self.client.on_session_enter(req).await?;

        Ok(())
    }

    async fn on_task_invoke(
        &mut self,
        ctx: &TaskContext,
    ) -> Result<Option<TaskOutput>, FlameError> {
        trace_fn!("GrpcShim::on_task_invoke");

        let req = Request::new(rpc::TaskContext::from(ctx.clone()));
        let resp = self.client.on_task_invoke(req).await?;
        let output = resp.into_inner();

        Ok(output.data.map(|d| d.into()))
    }

    async fn on_session_leave(&mut self) -> Result<(), FlameError> {
        trace_fn!("GrpcShim::on_session_leave");

        self.client
            .on_session_leave(Request::new(EmptyRequest::default()))
            .await?;

        Ok(())
    }
}
