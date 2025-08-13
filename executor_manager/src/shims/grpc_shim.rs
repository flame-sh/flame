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
use std::fs::{self, File};
use std::future::Future;
use std::pin::Pin;
use std::process::{self, Stdio};
use std::sync::Arc;
use std::task::{Context, Poll};
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
use common::apis::{ApplicationContext, SessionContext, TaskContext, TaskOutput};
use common::{trace::TraceFn, trace_fn, FlameError};

pub struct GrpcShim {
    session_context: Option<SessionContext>,
    client: GrpcShimClient<Channel>,
    child: tokio::process::Child,
    service_socket: String,
}

const RUST_LOG: &str = "RUST_LOG";
const DEFAULT_SVC_LOG_LEVEL: &str = "info";

impl GrpcShim {
    pub async fn new_ptr(app: &ApplicationContext) -> Result<ShimPtr, FlameError> {
        trace_fn!("GrpcShim::new_ptr");

        let command = app.command.clone().unwrap_or_default();
        let args = app.arguments.clone();
        let log_level = env::var(RUST_LOG).unwrap_or(String::from(DEFAULT_SVC_LOG_LEVEL));
        let mut envs = app.environments.clone();
        envs.insert(RUST_LOG.to_string(), log_level);

        log::debug!(
            "Try to start service by command <{command}> with args <{args:?}> and envs <{envs:?}>"
        );

        // Spawn child process
        let mut cmd = tokio::process::Command::new(&command);

        let mut child = cmd
            .envs(envs)
            .args(args)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                FlameError::InvalidConfig(format!(
                    "failed to start service by command <{command}>: {e}"
                ))
            })?;

        let service_id = child.id().unwrap_or_default();

        log::debug!("The service <{service_id}> was started, waiting for registering.");

        let service_socket = get_service_socket().await?;
        log::debug!("Try to connect to service <{service_id}> at <{service_socket}>");

        let channel = Endpoint::try_from("http://[::]:50051")
            .unwrap()
            .connect_with_connector({
                let service_addr = service_socket.clone();

                service_fn(move |_: Uri| {
                    let service_addr = service_addr.clone();
                    async move {
                        UnixStream::connect(service_addr)
                            .await
                            .map(TokioIo::new)
                            .map_err(std::io::Error::other)
                    }
                })
            })
            .await
            .map_err(|e| {
                FlameError::Network(format!("failed to connect to service <{service_id}>: {e}"))
            })?;

        let client = GrpcShimClient::new(channel);

        Ok(Arc::new(Mutex::new(Self {
            session_context: None,
            client,
            child,
            service_socket,
        })))
    }
}

impl Drop for GrpcShim {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = std::fs::remove_file(&self.service_socket);
        log::debug!(
            "The service <{}> was stopped",
            self.child.id().unwrap_or_default()
        );
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

async fn get_service_socket() -> Result<String, FlameError> {
    let path = "/tmp/flame/shim/fsi.sock".to_string();
    WaitForSvcSocketFuture::new(path).await
}

struct WaitForSvcSocketFuture {
    path: String,
}

impl WaitForSvcSocketFuture {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Future for WaitForSvcSocketFuture {
    type Output = Result<String, FlameError>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        if fs::exists(&self.path).unwrap_or(false) {
            Poll::Ready(Ok(self.path.clone()))
        } else {
            ctx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
