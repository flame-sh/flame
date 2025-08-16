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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use tonic::transport::Channel;

use self::rpc::backend_client::BackendClient as FlameBackendClient;
use self::rpc::{
    BindExecutorCompletedRequest, BindExecutorRequest, CompleteTaskRequest, LaunchTaskRequest,
    RegisterExecutorRequest, RegisterNodeRequest, ReleaseNodeRequest,
    UnbindExecutorCompletedRequest, UnbindExecutorRequest, UpdateNodeRequest,
    WatchAllocationRequest,
};
use ::rpc::flame as rpc;

use common::apis::{self, Node, ResourceRequirement, SessionContext, TaskContext};
use common::ctx::FlameContext;
use common::{lock_ptr, FlameError};
use crate::executor::Executor;

pub type FlameClient = FlameBackendClient<Channel>;

#[derive(Clone, Debug)]
pub struct BackendClient {
    client: FlameClient,
}

pub struct Allocation {
    pub replica: u32,
    pub resource_requirement: ResourceRequirement,
}

impl BackendClient {
    pub async fn new(ctx: &FlameContext) -> Result<Self, FlameError> {
        let client = FlameBackendClient::connect(ctx.endpoint.clone())
            .await
            .map_err(|_e| FlameError::Network("tonic connection".to_string()))?;

        Ok(Self { client })
    }

    pub async fn register_node(&mut self, node: &Node) -> Result<(), FlameError> {
        let req = RegisterNodeRequest {
            node: Some(node.clone().into()),
        };

        self.client
            .register_node(req)
            .await
            .map_err(FlameError::from)?;

        Ok(())
    }

    pub async fn update_node(&mut self, node: &Node) -> Result<(), FlameError> {
        let req = UpdateNodeRequest {
            node: Some(node.clone()),
        };

        self.client
            .update_node(req)
            .await
            .map_err(FlameError::from)?;

        Ok(())
    }

    pub async fn release_node(&mut self, node: &Node) -> Result<(), FlameError> {
        let req = ReleaseNodeRequest {
            node_name: node.name.clone(),
        };

        self.client
            .release_node(req)
            .await
            .map_err(FlameError::from)?;

        Ok(())
    }

    pub async fn watch_allocation(&mut self, node_name: &str) -> Result<Vec<Allocation>, FlameError> {
        let req = WatchAllocationRequest {
            node_name: node_name.to_string(),
        };

        let resp = self.client.watch_allocation(req).await?;
        let allocations = resp.into_inner().allocations;

        Ok(allocations)
    }

    pub async fn register_executor(&mut self, exe: &Executor) -> Result<(), FlameError> {
        let req = RegisterExecutorRequest {
            executor_id: exe.id.clone(),
            executor_spec: Some(rpc::ExecutorSpec { resreq: exe.resreq }),
        };

        self.client
            .register_executor(req)
            .await
            .map_err(FlameError::from)?;

        Ok(())
    }

    pub async fn bind_executor(&mut self, exe: &Executor) -> Result<SessionContext, FlameError> {
        let req = BindExecutorRequest {
            executor_id: exe.id.clone(),
        };

        let ssn = self
            .client
            .bind_executor(req)
            .await
            .map_err(FlameError::from)?;

        SessionContext::try_from(ssn.into_inner())
    }

    pub async fn bind_executor_completed(&mut self, exe: &Executor) -> Result<(), FlameError> {
        let req = BindExecutorCompletedRequest {
            executor_id: exe.id.clone(),
        };

        self.client
            .bind_executor_completed(req)
            .await
            .map_err(FlameError::from)?;

        Ok(())
    }

    //
    // rpc UnregisterExecutor (UnregisterExecutorRequest) returns (Result) {}
    //

    pub async fn unbind_executor(&mut self, exe: &Executor) -> Result<(), FlameError> {
        let req = UnbindExecutorRequest {
            executor_id: exe.id.clone(),
        };

        self.client
            .unbind_executor(req)
            .await
            .map_err(FlameError::from)?;
        Ok(())
    }

    pub async fn unbind_executor_completed(&mut self, exe: &Executor) -> Result<(), FlameError> {
        let req = UnbindExecutorCompletedRequest {
            executor_id: exe.id.clone(),
        };

        self.client
            .unbind_executor_completed(req)
            .await
            .map_err(FlameError::from)?;

        Ok(())
    }

    pub async fn launch_task(&mut self, exe: &Executor) -> Result<Option<TaskContext>, FlameError> {
        let req = LaunchTaskRequest {
            executor_id: exe.id.clone(),
        };

        let resp = self
            .client
            .launch_task(req)
            .await
            .map_err(FlameError::from)?;

        if let Some(t) = resp.into_inner().task {
            return Ok(Some(TaskContext::try_from(t)?));
        }

        Ok(None)
    }

    pub async fn complete_task(&mut self, exe: &Executor) -> Result<(), FlameError> {
        let task = exe
            .task
            .clone()
            .ok_or(FlameError::InvalidState("no task in executor".to_string()))?;

        let req = CompleteTaskRequest {
            executor_id: exe.id.clone(),
            task_output: task.output.map(apis::TaskOutput::into),
        };

        self.client
            .complete_task(req)
            .await
            .map_err(FlameError::from)?;

        Ok(())
    }
}
// rpc UnbindExecutor (UnbindExecutorRequest) returns (Result) {}
//
// rpc LaunchTask (LaunchTaskRequest) returns (Task) {}
// rpc CompleteTask(CompleteTaskRequest) returns (Result) {}
