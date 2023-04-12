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
use chrono::{DateTime, Utc};
use tonic::{Request, Response, Status};

use rpc::flame::frontend_server::Frontend;
use rpc::flame::{
    CreateSessionRequest, CreateTaskRequest, DeleteSessionRequest, DeleteTaskRequest,
    GetSessionRequest, GetTaskRequest, ListSessionRequest,
};

use rpc::flame::{Metadata, Session, SessionList, SessionSpec, SessionStatus, Task};

use crate::apiserver::Flame;
use crate::model;

#[async_trait]
impl Frontend for Flame {
    async fn create_session(
        &self,
        req: Request<CreateSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        let ssn_spec = req
            .into_inner()
            .session
            .ok_or(Status::invalid_argument("session spec"))?;

        let ssn = self
            .storage
            .create_session(ssn_spec.application, ssn_spec.slots)
            .map_err(Status::from)?;

        Ok(Response::new(Session::from(&ssn)))
    }

    async fn delete_session(
        &self,
        _: Request<DeleteSessionRequest>,
    ) -> Result<Response<rpc::flame::Result>, Status> {
        todo!()
    }

    async fn get_session(
        &self,
        req: Request<GetSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        let ssn_id = req
            .into_inner()
            .session_id
            .parse::<model::SessionID>()
            .map_err(|_| Status::invalid_argument("invalid session id"))?;

        let ssn = self.storage.get_session(ssn_id).map_err(Status::from)?;

        Ok(Response::new(Session::from(&ssn)))
    }
    async fn list_session(
        &self,
        _: Request<ListSessionRequest>,
    ) -> Result<Response<SessionList>, Status> {
        let ssn_list = self.storage.list_session().map_err(Status::from)?;

        let mut sessions = vec![];
        for ssn in &ssn_list {
            sessions.push(Session::from(ssn));
        }

        Ok(Response::new(SessionList { sessions }))
    }

    async fn create_task(&self, _: Request<CreateTaskRequest>) -> Result<Response<Task>, Status> {
        todo!()
    }
    async fn delete_task(
        &self,
        _: Request<DeleteTaskRequest>,
    ) -> Result<Response<rpc::flame::Result>, Status> {
        todo!()
    }
    async fn get_task(&self, _: Request<GetTaskRequest>) -> Result<Response<Task>, Status> {
        todo!()
    }
}

impl From<&model::Session> for Session {
    fn from(ssn: &model::Session) -> Self {
        let mut status = SessionStatus {
            state: 0,
            creation_time: ssn.creation_time.timestamp(),
            completion_time: match ssn.completion_time {
                None => None,
                Some(s) => Some(s.timestamp()),
            },
            failed: 0,
            pending: 0,
            running: 0,
            succeed: 0,
        };
        for (s, v) in &ssn.tasks_index {
            match s {
                model::TaskState::Pending => status.pending = v.len() as i32,
                model::TaskState::Running => status.running = v.len() as i32,
                model::TaskState::Completed => status.succeed = v.len() as i32,
                model::TaskState::Failed => status.failed = v.len() as i32,
            }
        }

        Session {
            metadata: Some(Metadata {
                id: ssn.id.to_string(),
                owner: None,
            }),
            spec: Some(SessionSpec {
                application: ssn.application.clone(),
                slots: ssn.slots,
            }),
            status: Some(status),
        }
    }
}
