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

use async_trait::async_trait;
use chrono::Utc;
use common::{trace::TraceFn, trace_fn, FlameError};
use tonic::{Request, Response, Status};

use self::rpc::backend_server::Backend;
use self::rpc::{
    Application, BindExecutorCompletedRequest, BindExecutorRequest, BindExecutorResponse,
    CompleteTaskRequest, LaunchTaskRequest, LaunchTaskResponse, RegisterExecutorRequest,
    RegisterNodeRequest, ReleaseNodeRequest, Session, SyncNodeRequest, SyncNodeResponse,
    UnbindExecutorCompletedRequest, UnbindExecutorRequest, UnregisterExecutorRequest,
};
use ::rpc::flame as rpc;

use crate::apiserver::Flame;
use common::apis::{Executor, ExecutorState};
use common::apis::Node;
use common::apis::TaskOutput;

#[async_trait]
impl Backend for Flame {
    async fn register_node(
        &self,
        req: Request<RegisterNodeRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        trace_fn!("Backend::register_node");
        let req = req.into_inner();
        let node = Node::from(
            req.node
                .ok_or(FlameError::InvalidConfig("node is required".to_string()))?,
        );
        self.controller.register_node(&node).await?;
        Ok(Response::new(rpc::Result::default()))
    }
    async fn sync_node(
        &self,
        req: Request<SyncNodeRequest>,
    ) -> Result<Response<SyncNodeResponse>, Status> {
        trace_fn!("Backend::sync_node");
        let req = req.into_inner();
        let node = Node::from(
            req.node
                .ok_or(FlameError::InvalidConfig("node is required".to_string()))?,
        );
        let executors: Vec<Executor> = req.executors.into_iter().map(rpc::Executor::into).collect();

        let executors = self.controller.sync_node(&node, &executors).await?;

        Ok(Response::new(SyncNodeResponse {
            node: Some(node.into()),
            executors: executors.into_iter().map(rpc::Executor::into).collect(),
        }))
    }

    async fn release_node(
        &self,
        req: Request<ReleaseNodeRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        trace_fn!("Backend::release_node");
        let req = req.into_inner();
        self.controller.release_node(&req.node_name).await?;
        Ok(Response::new(rpc::Result::default()))
    }

    async fn register_executor(
        &self,
        req: Request<RegisterExecutorRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        trace_fn!("Backend::register_executor");
        let req = req.into_inner();
        let spec = req
            .executor_spec
            .ok_or(FlameError::InvalidConfig("no executor spec".to_string()))?;

        let e = Executor {
            id: req.executor_id,
            resreq: spec.resreq.unwrap_or_default().into(),
            task_id: None,
            ssn_id: None,
            creation_time: Utc::now(),
            state: ExecutorState::Idle,
        };

        self.controller
            .register_executor(&e)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(rpc::Result::default()))
    }
    async fn unregister_executor(
        &self,
        _: Request<UnregisterExecutorRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        todo!()
    }

    async fn bind_executor(
        &self,
        req: Request<BindExecutorRequest>,
    ) -> Result<Response<BindExecutorResponse>, Status> {
        trace_fn!("Backend::bind_executor");
        let req = req.into_inner();

        let ssn = self
            .controller
            .wait_for_session(req.executor_id.to_string())
            .await?;
        let session = Some(Session::from(&ssn));

        let app = self.controller.get_application(ssn.application).await?;
        let application = Some(Application::from(&app));

        log::debug!(
            "Bind executor <{}> to Session <{}:{}>",
            req.executor_id.to_string(),
            app.name,
            ssn.id
        );

        Ok(Response::new(BindExecutorResponse {
            application,
            session,
        }))
    }

    async fn bind_executor_completed(
        &self,
        req: Request<BindExecutorCompletedRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        trace_fn!("Backend::bind_executor_completed");
        let req = req.into_inner();

        self.controller
            .bind_session_completed(req.executor_id)
            .await?;

        Ok(Response::new(rpc::Result::default()))
    }

    async fn unbind_executor(
        &self,
        req: Request<UnbindExecutorRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        let req = req.into_inner();
        self.controller.unbind_executor(req.executor_id).await?;

        Ok(Response::new(rpc::Result::default()))
    }

    async fn unbind_executor_completed(
        &self,
        req: Request<UnbindExecutorCompletedRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        let req = req.into_inner();
        self.controller
            .unbind_executor_completed(req.executor_id)
            .await?;

        Ok(Response::new(rpc::Result::default()))
    }

    async fn launch_task(
        &self,
        req: Request<LaunchTaskRequest>,
    ) -> Result<Response<LaunchTaskResponse>, Status> {
        let req = req.into_inner();
        let task = self.controller.launch_task(req.executor_id).await?;
        if let Some(task) = task {
            return Ok(Response::new(LaunchTaskResponse {
                task: Some(rpc::Task::from(&task)),
            }));
        }

        Ok(Response::new(LaunchTaskResponse { task: None }))
    }

    async fn complete_task(
        &self,
        req: Request<CompleteTaskRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        let req = req.into_inner();

        self.controller
            .complete_task(
                req.executor_id.clone(),
                req.task_output.map(TaskOutput::from),
            )
            .await?;

        Ok(Response::new(rpc::Result::default()))
    }
}
