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
use std::pin::Pin;

use ::rpc::flame::{
    ApplicationList, GetApplicationRequest, ListApplicationRequest, RegisterApplicationRequest,
    UnregisterApplicationRequest, UpdateApplicationRequest,
};
use async_trait::async_trait;
use common::apis::ApplicationAttributes;
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use self::rpc::frontend_server::Frontend;
use self::rpc::{
    CloseSessionRequest, CreateSessionRequest, CreateTaskRequest, DeleteSessionRequest,
    DeleteTaskRequest, GetSessionRequest, GetTaskRequest, ListSessionRequest, OpenSessionRequest,
    Session, SessionList, Task, WatchTaskRequest,
};
use rpc::flame as rpc;

use common::{apis, FlameError};
use common::{trace::TraceFn, trace_fn};

use crate::apiserver::Flame;

#[async_trait]
impl Frontend for Flame {
    type WatchTaskStream = Pin<Box<dyn Stream<Item = Result<Task, Status>> + Send>>;

    async fn register_application(
        &self,
        req: Request<RegisterApplicationRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        trace_fn!("Frontend::register_application");

        let req = req.into_inner();
        let spec = req.application.ok_or(FlameError::InvalidConfig(
            "applilcation spec is missed".to_string(),
        ))?;
        let res = self
            .controller
            .register_application(req.name, ApplicationAttributes::from(spec))
            .await;

        match res {
            Ok(..) => Ok(Response::new(rpc::Result {
                return_code: 0,
                message: None,
            })),
            Err(e) => Ok(Response::new(rpc::Result {
                return_code: -1,
                message: Some(e.to_string()),
            })),
        }
    }
    async fn unregister_application(
        &self,
        req: Request<UnregisterApplicationRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        todo!()
    }

    async fn update_application(
        &self,
        req: Request<UpdateApplicationRequest>,
    ) -> Result<Response<rpc::Result>, Status> {
        todo!()
    }

    async fn get_application(
        &self,
        req: tonic::Request<GetApplicationRequest>,
    ) -> Result<Response<rpc::Application>, Status> {
        todo!()
    }

    async fn list_application(
        &self,
        _: Request<ListApplicationRequest>,
    ) -> Result<Response<ApplicationList>, Status> {
        trace_fn!("Frontend::list_application");
        let app_list = self
            .controller
            .list_application()
            .await
            .map_err(Status::from)?;

        let applications = app_list.iter().map(rpc::Application::from).collect();

        Ok(Response::new(ApplicationList { applications }))
    }

    async fn create_session(
        &self,
        req: Request<CreateSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        trace_fn!("Frontend::create_session");
        let ssn_spec = req
            .into_inner()
            .session
            .ok_or(Status::invalid_argument("session spec"))?;

        let ssn = self
            .controller
            .create_session(
                ssn_spec.application,
                ssn_spec.slots,
                ssn_spec.common_data.map(apis::CommonData::from),
            )
            .await
            .map(Session::from)
            .map_err(Status::from)?;

        Ok(Response::new(ssn))
    }

    async fn delete_session(
        &self,
        req: Request<DeleteSessionRequest>,
    ) -> Result<Response<rpc::Session>, Status> {
        let ssn_id = req
            .into_inner()
            .session_id
            .parse::<apis::SessionID>()
            .map_err(|_| Status::invalid_argument("invalid session id"))?;

        let ssn = self
            .controller
            .delete_session(ssn_id)
            .await
            .map(Session::from)?;

        Ok(Response::new(ssn))
    }

    async fn open_session(
        &self,
        _: Request<OpenSessionRequest>,
    ) -> Result<Response<rpc::Session>, Status> {
        todo!()
    }

    async fn close_session(
        &self,
        req: Request<CloseSessionRequest>,
    ) -> Result<Response<rpc::Session>, Status> {
        trace_fn!("Frontend::close_session");
        let ssn_id = req
            .into_inner()
            .session_id
            .parse::<apis::SessionID>()
            .map_err(|_| Status::invalid_argument("invalid session id"))?;

        let ssn = self
            .controller
            .close_session(ssn_id)
            .await
            .map(rpc::Session::from)
            .map_err(Status::from)?;

        Ok(Response::new(ssn))
    }

    async fn get_session(
        &self,
        req: Request<GetSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        trace_fn!("Frontend::get_session");
        let ssn_id = req
            .into_inner()
            .session_id
            .parse::<apis::SessionID>()
            .map_err(|_| Status::invalid_argument("invalid session id"))?;

        let ssn = self
            .controller
            .get_session(ssn_id)
            .map(rpc::Session::from)
            .map_err(Status::from)?;

        Ok(Response::new(ssn))
    }
    async fn list_session(
        &self,
        _: Request<ListSessionRequest>,
    ) -> Result<Response<SessionList>, Status> {
        trace_fn!("Frontend::list_session");
        let ssn_list = self.controller.list_session().map_err(Status::from)?;

        let sessions = ssn_list.iter().map(Session::from).collect();

        Ok(Response::new(SessionList { sessions }))
    }

    async fn create_task(&self, req: Request<CreateTaskRequest>) -> Result<Response<Task>, Status> {
        trace_fn!("Frontend::create_task");
        let task_spec = req
            .into_inner()
            .task
            .ok_or(Status::invalid_argument("session spec"))?;
        let ssn_id = task_spec
            .session_id
            .parse::<apis::SessionID>()
            .map_err(|_| Status::invalid_argument("invalid session id"))?;

        let task = self
            .controller
            .create_task(ssn_id, task_spec.input.map(apis::TaskInput::from))
            .await
            .map(Task::from)
            .map_err(Status::from)?;

        Ok(Response::new(task))
    }
    async fn delete_task(
        &self,
        _: Request<DeleteTaskRequest>,
    ) -> Result<Response<rpc::Task>, Status> {
        todo!()
    }

    async fn watch_task(
        &self,
        req: Request<WatchTaskRequest>,
    ) -> Result<Response<Self::WatchTaskStream>, Status> {
        let req = req.into_inner();
        let gid = apis::TaskGID {
            ssn_id: req
                .session_id
                .parse::<apis::SessionID>()
                .map_err(|_| Status::invalid_argument("invalid session id"))?,

            task_id: req
                .task_id
                .parse::<apis::SessionID>()
                .map_err(|_| Status::invalid_argument("invalid task id"))?,
        };

        let (tx, rx) = mpsc::channel(128);

        let controller = self.controller.clone();
        tokio::spawn(async move {
            loop {
                match controller.watch_task(gid).await {
                    Ok(task) => {
                        log::debug!("Task <{}> state is <{}>", task.id, task.state as i32);
                        if let Err(e) = tx.send(Result::<_, Status>::Ok(Task::from(&task))).await {
                            log::debug!("Failed to send Task <{}>: {}", gid, e);
                            break;
                        }
                        if task.is_completed() {
                            log::debug!("Task <{}> is completed, exit.", task.id);
                            break;
                        }
                    }
                    Err(e) => {
                        log::debug!("Failed to watch Task <{}>: {}", gid, e);
                        break;
                    }
                }
            }
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::WatchTaskStream
        ))
    }

    async fn get_task(&self, req: Request<GetTaskRequest>) -> Result<Response<Task>, Status> {
        let req = req.into_inner();
        let ssn_id = req
            .session_id
            .parse::<apis::SessionID>()
            .map_err(|_| Status::invalid_argument("invalid session id"))?;

        let task_id = req
            .task_id
            .parse::<apis::SessionID>()
            .map_err(|_| Status::invalid_argument("invalid task id"))?;

        let task = self
            .controller
            .get_task(ssn_id, task_id)
            .map(Task::from)
            .map_err(Status::from)?;

        Ok(Response::new(task))
    }
}
