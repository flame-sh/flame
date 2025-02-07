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

use std::sync::Arc;
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

use crate::shims::{Shim, ShimPtr};
use common::apis::{ApplicationContext, SessionContext, TaskContext, TaskOutput};
use common::FlameError;

pub struct GrpcShim {
    session_context: Option<SessionContext>,
    client: GrpcShimClient<Channel>,
    child: tokio::process::Child,
}

const FLAME_SOCKET_PATH: &str = "FLAME_SOCKET_PATH";

impl GrpcShim {
    pub async fn new_ptr(app_ctx: &ApplicationContext) -> Result<ShimPtr, FlameError> {
        let socket_path = format!("/tmp/flame-shim-{}.sock", uuid::Uuid::new_v4().simple());
        std::env::set_var(FLAME_SOCKET_PATH, socket_path.clone());

        // Spawn child process
        let mut cmd = tokio::process::Command::new(&app_ctx.command.clone().unwrap());
        cmd.env(FLAME_SOCKET_PATH, &socket_path).kill_on_drop(true);

        let child = cmd
            .env(FLAME_SOCKET_PATH, &socket_path)
            .spawn()
            .map_err(|e| FlameError::InvalidConfig(e.to_string()))?;

        let channel = Endpoint::try_from("http://[::]:50051")
            .map_err(|e| FlameError::Network(e.to_string()))?
            .connect_with_connector(service_fn(|_: Uri| async {
                // Connect to a Uds socket
                let path = std::env::var(FLAME_SOCKET_PATH).ok().unwrap();
                Ok::<_, std::io::Error>(TokioIo::new(UnixStream::connect(path).await?))
            }))
            .await
            .map_err(|e| FlameError::Network(e.to_string()))?;

        let mut client = GrpcShimClient::new(channel);

        let mut connected = false;
        for i in 1..10 {
            let resp = client
                .readiness(Request::new(EmptyRequest::default()))
                .await;
            if resp.is_ok() {
                connected = true;
                break;
            }
            // sleep 1s
            let ten_millis = time::Duration::from_secs(1);
            thread::sleep(ten_millis);
        }

        if !connected {
            return Err(FlameError::InvalidConfig(
                "failed to connect to service".to_string(),
            ));
        }

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
        let req = Request::new(rpc::SessionContext::from(ctx.clone()));
        self.client.on_session_enter(req).await?;
        Ok(())
    }

    async fn on_task_invoke(
        &mut self,
        ctx: &TaskContext,
    ) -> Result<Option<TaskOutput>, FlameError> {
        let req = Request::new(rpc::TaskContext::from(ctx.clone()));
        let resp = self.client.on_task_invoke(req).await?;
        let output = resp.into_inner();

        Ok(output.data.map(|d| d.into()))
    }

    async fn on_session_leave(&mut self) -> Result<(), FlameError> {
        let _ = self
            .client
            .on_session_leave(Request::new(EmptyRequest::default()))
            .await?;

        self.child.kill();

        Ok(())
    }
}
