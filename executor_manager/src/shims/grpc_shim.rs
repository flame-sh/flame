/*
Copyright 2024 The Flame Authors.
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
use std::path::{Path, MAIN_SEPARATOR};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use self::rpc::instance_client::InstanceClient as FlameInstanceClient;
use ::rpc::flame as rpc;
use ::rpc::flame::{OnSessionEnterRequest, OnSessionLeaveRequest, OnTaskInvokeRequest};

use crate::shims::{Shim, ShimPtr};
use common::apis::{Application, SessionContext, TaskContext, TaskOutput};
use common::FlameError;

#[derive(Clone)]
pub struct GrpcShim {
    instance: Arc<Mutex<Child>>,
    instance_client: FlameInstanceClient<Channel>,
}

impl GrpcShim {
    pub async fn new_ptr(app: &Application) -> Result<ShimPtr, FlameError> {
        let mut cmd = app.command.clone();
        let path = Path::new(&cmd);
        if !path.has_root() {
            match env::current_dir() {
                Ok(cwd) => match cwd.to_str() {
                    None => {
                        log::warn!("Failed to get current directory path string.");
                    }
                    Some(cwd) => {
                        cmd = format!("{}{}{}", cwd, MAIN_SEPARATOR, cmd);
                    }
                },
                Err(e) => {
                    log::warn!(
                        "Failed to get current directory for application <{}>: {}",
                        &cmd,
                        e
                    );
                }
            }
        }

        let instance = Command::new(&cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(&app.working_directory)
            .spawn()
            .map_err(|_| FlameError::Internal("failed to start instance".to_string()))?;

        let instance_client = FlameInstanceClient::connect("unix://var/flame/shim.sock")
            .await
            .map_err(|e| FlameError::Network(e.to_string()))?;

        Ok(Arc::new(Mutex::new(Self {
            instance: Arc::new(Mutex::new(instance)),
            instance_client,
        })))
    }
}

#[async_trait]
impl Shim for GrpcShim {
    async fn on_session_enter(&mut self, ctx: &SessionContext) -> Result<(), FlameError> {
        let req = OnSessionEnterRequest {
            session_id: ctx.ssn_id.clone(),
            common_data: ctx.common_data.clone().map(Bytes::into),
        };

        self.instance_client.on_session_enter(req).await?;

        Ok(())
    }

    async fn on_task_invoke(
        &mut self,
        ctx: &TaskContext,
    ) -> Result<Option<TaskOutput>, FlameError> {
        let req = OnTaskInvokeRequest {
            task_id: ctx.id.clone(),
            session_id: ctx.ssn_id.clone(),
            input: ctx.input.clone().map(Bytes::into),
        };

        let resp = self.instance_client.on_task_invoke(req).await?.into_inner();

        Ok(resp.output.map(Bytes::from))
    }

    async fn on_session_leave(&mut self) -> Result<(), FlameError> {
        self.instance_client
            .on_session_leave(OnSessionLeaveRequest {})
            .await?;

        let mut instance = self.instance.lock().await;
        instance
            .wait()
            .map_err(|e| FlameError::Internal(e.to_string()))?;
        Ok(())
    }
}
