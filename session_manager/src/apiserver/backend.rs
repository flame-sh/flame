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

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use rpc::flame::{BindExecutorRequest, CompleteTaskRequest, Executor, LaunchTaskRequest, RegisterExecutorRequest, Session, Task, UnbindExecutorRequest, UnregisterExecutorRequest};
use rpc::flame::backend_server::Backend;

use crate::apiserver::Flame;

#[async_trait]
impl Backend for Flame {
    async fn register_executor(&self, _: Request<RegisterExecutorRequest>) -> Result<Response<Executor>, Status> { todo!() }
    async fn unregister_executor(&self, _: Request<UnregisterExecutorRequest>) -> Result<Response<rpc::flame::Result>, Status> { todo!() }
    async fn bind_executor(&self, _: Request<BindExecutorRequest>) -> Result<Response<Session>, Status> { todo!() }
    async fn unbind_executor(&self, _: Request<UnbindExecutorRequest>) -> Result<Response<rpc::flame::Result>, Status> { todo!() }
    async fn launch_task(&self, _: Request<LaunchTaskRequest>) -> Result<Response<Task>, Status> { todo!() }
    async fn complete_task(&self, _: Request<CompleteTaskRequest>) -> Result<Response<rpc::flame::Result>, Status> { todo!() }
}