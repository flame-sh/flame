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

use std::{process, sync::Arc};

use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{
    transport::Server,
    Request, Response, Status,
};

use self::rpc::grpc_shim_server::{GrpcShim, GrpcShimServer};
use crate::apis::flame as rpc;

use crate::apis::{CommonData, FlameError, TaskInput, TaskOutput};

pub struct ApplicationContext {
    pub name: String,
    pub url: Option<String>,
    pub command: Option<String>,
}

pub struct SessionContext {
    pub session_id: String,
    pub application: ApplicationContext,
    pub common_data: Option<CommonData>,
}

pub struct TaskContext {
    pub task_id: String,
    pub session_id: String,
    pub input: Option<TaskInput>,
}

#[tonic::async_trait]
pub trait FlameService: Send + Sync + 'static {
    async fn on_session_enter(&self, _: SessionContext) -> Result<(), FlameError>;
    async fn on_task_invoke(&self, _: TaskContext) -> Result<Option<TaskOutput>, FlameError>;
    async fn on_session_leave(&self) -> Result<(), FlameError>;
}

pub type FlameServicePtr = Arc<dyn FlameService>;

struct ShimService {
    service: FlameServicePtr,
}

#[tonic::async_trait]
impl GrpcShim for ShimService {
    async fn on_session_enter(
        &self,
        req: Request<rpc::SessionContext>,
    ) -> Result<Response<rpc::Result>, Status> {
        log::debug!("ShimService::on_session_enter");

        let req = req.into_inner();
        self.service
            .on_session_enter(SessionContext::from(req))
            .await?;

        Ok(Response::new(rpc::Result {
            return_code: 0,
            message: None,
        }))
    }

    async fn on_task_invoke(
        &self,
        req: Request<rpc::TaskContext>,
    ) -> Result<Response<rpc::TaskOutput>, Status> {
        log::debug!("ShimService::on_task_invoke");
        let req = req.into_inner();
        let data = self.service.on_task_invoke(TaskContext::from(req)).await?;

        Ok(Response::new(rpc::TaskOutput {
            data: data.map(|d| d.into()),
        }))
    }

    async fn on_session_leave(
        &self,
        _: Request<rpc::EmptyRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        log::debug!("ShimService::on_session_leave");
        self.service.on_session_leave().await?;

        Ok(Response::new(rpc::Result {
            return_code: 0,
            message: None,
        }))
    }
}

pub async fn run(service: impl FlameService) -> Result<(), Box<dyn std::error::Error>> {
    let shim_service = ShimService {
        service: Arc::new(service),
    };

    let service_id = process::id().to_string();

    let uds = UnixListener::bind(format!("/tmp/flame/shim/{service_id}.sock"))?;
    let uds_stream = UnixListenerStream::new(uds);

    Server::builder()
        .add_service(GrpcShimServer::new(shim_service))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}

impl From<FlameError> for Status {
    fn from(e: FlameError) -> Self {
        Status::internal(e.to_string())
    }
}

impl From<rpc::ApplicationContext> for ApplicationContext {
    fn from(ctx: rpc::ApplicationContext) -> Self {
        Self {
            name: ctx.name.clone(),
            url: ctx.url.clone(),
            command: ctx.command.clone(),
        }
    }
}

impl From<rpc::SessionContext> for SessionContext {
    fn from(ctx: rpc::SessionContext) -> Self {
        SessionContext {
            session_id: ctx.session_id.clone(),
            application: ctx.application.map(ApplicationContext::from).unwrap(),
            common_data: ctx.common_data.map(|data| data.into()),
        }
    }
}

impl From<rpc::TaskContext> for TaskContext {
    fn from(ctx: rpc::TaskContext) -> Self {
        TaskContext {
            task_id: ctx.task_id.clone(),
            session_id: ctx.session_id.clone(),
            input: ctx.input.map(|data| data.into()),
        }
    }
}
