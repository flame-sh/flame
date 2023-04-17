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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use tonic::transport::Channel;

use self::rpc::backend_client::BackendClient as FlameBackendClient;
use self::rpc::{BindExecutorRequest, RegisterExecutorRequest, BindExecutorCompletedRequest};
use ::rpc::flame as rpc;

use crate::executor::{Executor, SessionContext};
use common::{lock_ptr, FlameContext, FlameError};

type FlameClient = FlameBackendClient<Channel>;

#[derive(Clone, Debug)]
pub struct BackendClient {
    client_pool: Arc<Mutex<HashMap<String, FlameClient>>>,
}

lazy_static! {
    static ref INSTANCE: Arc<BackendClient> = Arc::new(BackendClient {
        client_pool: Arc::new(Mutex::new(HashMap::new()))
    });
}

pub async fn install(ctx: &FlameContext) -> Result<(), FlameError> {
    let client = FlameBackendClient::connect(ctx.endpoint.clone())
        .await
        .map_err(|_e| FlameError::Network("tonic connection".to_string()))?;

    let mut cs = lock_ptr!(INSTANCE.client_pool)?;
    cs.insert(ctx.name.clone(), client);

    Ok(())
}

fn get_client(ctx: &FlameContext) -> Result<FlameClient, FlameError> {
    let cs = lock_ptr!(INSTANCE.client_pool)?;
    let client = cs.get(&ctx.name).ok_or(FlameError::Uninitialized(format!(
        "client {}",
        ctx.name.clone()
    )))?;

    Ok(client.clone())
}

pub async fn register_executor(ctx: &FlameContext, exe: &Executor) -> Result<(), FlameError> {
    let mut ins = get_client(ctx)?;

    let req = RegisterExecutorRequest {
        executor_id: exe.id.clone(),
        executor_spec: Some(rpc::ExecutorSpec::from(exe)),
    };

    ins.register_executor(req).await.map_err(FlameError::from)?;

    Ok(())
}

pub async fn bind_executor(
    ctx: &FlameContext,
    exe: &Executor,
) -> Result<SessionContext, FlameError> {
    let mut ins = get_client(ctx)?;

    let req = BindExecutorRequest {
        executor_id: exe.id.clone(),
    };

    let ssn = ins.bind_executor(req).await.map_err(FlameError::from)?;
    Ok(SessionContext::try_from(ssn.into_inner())?)
}

pub async fn bind_executor_completed(ctx: &FlameContext, exe: &Executor) -> Result<(), FlameError> {
    let mut ins = get_client(ctx)?;

    let req = BindExecutorCompletedRequest {
        executor_id: exe.id.clone(),
    };

    let ssn = ins
        .bind_executor_completed(req)
        .await
        .map_err(FlameError::from)?;
    Ok(())
}

// rpc BindExecutor (BindExecutorRequest) returns (Session) {}

//
// rpc UnregisterExecutor (UnregisterExecutorRequest) returns (Result) {}
//
// rpc BindExecutor (BindExecutorRequest) returns (Session) {}
// rpc UnbindExecutor (UnbindExecutorRequest) returns (Result) {}
//
// rpc LaunchTask (LaunchTaskRequest) returns (Task) {}
// rpc CompleteTask(CompleteTaskRequest) returns (Result) {}
