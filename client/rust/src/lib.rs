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

use std::sync::{Arc, Mutex};

use bytes::Bytes;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::TryFutureExt;
use prost::Enumeration;
use thiserror::Error;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tonic::transport::Endpoint;
use tonic::Status;

use self::rpc::frontend_client::FrontendClient as FlameFrontendClient;
use self::rpc::{
    CloseSessionRequest, CreateSessionRequest, CreateTaskRequest, GetTaskRequest,
    ListSessionRequest, SessionSpec, TaskSpec, WatchTaskRequest,
};
use crate::flame as rpc;
use crate::trace::TraceFn;

mod trace;

mod flame {
    tonic::include_proto!("flame");
}

type FlameClient = FlameFrontendClient<Channel>;
type TaskID = String;
type SessionID = String;

type Message = Bytes;
pub type TaskInput = Message;
pub type TaskOutput = Message;

#[macro_export]
macro_rules! lock_ptr {
    ( $mutex_arc:expr ) => {
        $mutex_arc
            .lock()
            .map_err(|_| FlameError::Internal("mutex ptr".to_string()))
    };
}

pub async fn connect(addr: &str) -> Result<Connection, FlameError> {
    let endpoint = Endpoint::from_shared(addr.to_string())
        .map_err(|_| FlameError::InvalidConfig("invalid address".to_string()))?;

    let channel = endpoint
        .connect()
        .await
        .map_err(|_| FlameError::InvalidConfig("failed to connect".to_string()))?;

    Ok(Connection { channel })
}

#[derive(Error, Debug, Clone)]
pub enum FlameError {
    #[error("'{0}' not found")]
    NotFound(String),

    #[error("'{0}'")]
    Internal(String),

    #[error("'{0}'")]
    Network(String),

    #[error("'{0}'")]
    InvalidConfig(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration, strum_macros::Display)]
pub enum SessionState {
    Open = 0,
    Closed = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration, strum_macros::Display)]
pub enum TaskState {
    Pending = 0,
    Running = 1,
    Succeed = 2,
    Failed = 3,
}

#[derive(Clone)]
pub struct Connection {
    pub(crate) channel: Channel,
}

#[derive(Clone)]
pub struct SessionAttributes {
    pub application: String,
    pub slots: i32,
}

#[derive(Clone)]
pub struct Session {
    pub(crate) client: Option<FlameClient>,

    pub id: SessionID,
    pub slots: i32,
    pub application: String,
    pub creation_time: DateTime<Utc>,

    pub state: SessionState,
    pub pending: i32,
    pub running: i32,
    pub succeed: i32,
    pub failed: i32,
}

#[derive(Clone)]
pub struct Task {
    pub id: TaskID,
    pub ssn_id: SessionID,

    pub state: TaskState,

    pub input: Option<TaskInput>,
    pub output: Option<TaskOutput>,
}

pub type TaskInformerPtr = Arc<Mutex<dyn TaskInformer>>;
pub type TaskResultPtr = Arc<Mutex<Result<Task, FlameError>>>;

pub trait TaskInformer: Send + Sync + 'static {
    fn on_update(&mut self, task: Task);
    fn on_error(&mut self, e: FlameError);
}

impl Task {
    pub fn is_completed(&self) -> bool {
        self.state == TaskState::Succeed || self.state == TaskState::Failed
    }
}

impl Connection {
    pub async fn create_session(&self, attrs: &SessionAttributes) -> Result<Session, FlameError> {
        trace_fn!("Connection::create_session");

        let create_ssn_req = CreateSessionRequest {
            session: Some(SessionSpec {
                application: attrs.application.clone(),
                slots: attrs.slots,
            }),
        };

        let mut client = FlameClient::new(self.channel.clone());
        let ssn = client.create_session(create_ssn_req).await?;
        let ssn = ssn.into_inner();

        let mut ssn = Session::from(&ssn);
        ssn.client = Some(client);

        Ok(ssn)
    }

    pub async fn list_session(&self) -> Result<Vec<Session>, FlameError> {
        let mut client = FlameClient::new(self.channel.clone());
        let ssn_list = client.list_session(ListSessionRequest {}).await?;

        Ok(ssn_list
            .into_inner()
            .sessions
            .iter()
            .map(Session::from)
            .collect())
    }
}

impl Session {
    pub async fn create_task(&self, input: Option<TaskInput>) -> Result<Task, FlameError> {
        trace_fn!("Session::create_task");
        let mut client = self
            .client
            .clone()
            .ok_or(FlameError::Internal("no flame client".to_string()))?;

        let create_task_req = CreateTaskRequest {
            task: Some(TaskSpec {
                session_id: self.id.clone(),
                input: input.map(|input| input.to_vec()),
                output: None,
            }),
        };

        let task = client.create_task(create_task_req).await?;

        let task = task.into_inner();
        Ok(Task::from(&task))
    }

    pub async fn get_task(&self, id: TaskID) -> Result<Task, FlameError> {
        trace_fn!("Session::get_task");
        let mut client = self
            .client
            .clone()
            .ok_or(FlameError::Internal("no flame client".to_string()))?;

        let get_task_req = GetTaskRequest {
            session_id: self.id.clone(),
            task_id: id.clone(),
        };
        let task = client.get_task(get_task_req).await?;

        let task = task.into_inner();
        Ok(Task::from(&task))
    }

    pub async fn run_task(
        &self,
        input: Option<TaskInput>,
        informer_ptr: TaskInformerPtr,
    ) -> Result<(), FlameError> {
        trace_fn!("Session::run_task");
        self.create_task(input)
            .and_then(|task| self.watch_task(task.ssn_id.clone(), task.id, informer_ptr))
            .await
    }

    pub async fn watch_task(
        &self,
        session_id: SessionID,
        task_id: TaskID,
        informer_ptr: TaskInformerPtr,
    ) -> Result<(), FlameError> {
        trace_fn!("Session::watch_task");
        let mut client = self
            .client
            .clone()
            .ok_or(FlameError::Internal("no flame client".to_string()))?;

        let watch_task_req = WatchTaskRequest {
            session_id,
            task_id,
        };
        let mut task_stream = client.watch_task(watch_task_req).await?.into_inner();
        while let Some(task) = task_stream.next().await {
            match task {
                Ok(t) => {
                    let mut informer = lock_ptr!(informer_ptr)?;
                    informer.on_update(Task::from(&t));
                }
                Err(e) => {
                    let mut informer = lock_ptr!(informer_ptr)?;
                    informer.on_error(FlameError::from(e.clone()));
                }
            }
        }
        Ok(())
    }

    pub async fn close(&self) -> Result<(), FlameError> {
        trace_fn!("Session::close");
        let mut client = self
            .client
            .clone()
            .ok_or(FlameError::Internal("no flame client".to_string()))?;

        let close_ssn_req = CloseSessionRequest {
            session_id: self.id.clone(),
        };

        client.close_session(close_ssn_req).await?;

        Ok(())
    }
}

impl From<Status> for FlameError {
    fn from(value: Status) -> Self {
        FlameError::Network(value.code().to_string())
    }
}

impl From<&rpc::Task> for Task {
    fn from(task: &rpc::Task) -> Self {
        let metadata = task.metadata.clone().unwrap();
        let spec = task.spec.clone().unwrap();
        let status = task.status.clone().unwrap();
        Task {
            id: metadata.id,
            ssn_id: spec.session_id.clone(),
            input: spec.input.map(TaskInput::from),
            output: spec.output.map(TaskOutput::from),
            state: TaskState::try_from(status.state).unwrap_or(TaskState::default()),
        }
    }
}

impl From<&rpc::Session> for Session {
    fn from(ssn: &rpc::Session) -> Self {
        let metadata = ssn.metadata.clone().unwrap();
        let status = ssn.status.clone().unwrap();
        let spec = ssn.spec.clone().unwrap();

        let naivedatetime_utc =
            NaiveDateTime::from_timestamp_millis(status.creation_time * 1000).unwrap();
        let creation_time = DateTime::<Utc>::from_utc(naivedatetime_utc, Utc);

        Session {
            client: None,
            id: metadata.id,
            slots: spec.slots,
            application: spec.application,
            creation_time,
            state: SessionState::try_from(status.state).unwrap_or(SessionState::default()),
            pending: status.pending,
            running: status.running,
            succeed: status.succeed,
            failed: status.failed,
        }
    }
}
