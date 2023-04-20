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

use bytes::Bytes;
use lazy_static::lazy_static;
use prost::Enumeration;
use thiserror::Error;
use tonic::transport::Channel;
use tonic::Status;

use self::rpc::frontend_client::FrontendClient as FlameFrontendClient;
use self::rpc::{
    CloseSessionRequest, CreateSessionRequest, CreateTaskRequest, GetTaskRequest, SessionSpec,
    TaskSpec,
};
use crate::TaskState::{Failed, Succeed};
use ::rpc::flame as rpc;

type FlameClient = FlameFrontendClient<Channel>;
type TaskID = String;
type SessionID = String;

type Message = Bytes;
pub type TaskInput = Message;
pub type TaskOutput = Message;

const FLAME_CLIENT_NAME: &str = "flame";

macro_rules! lock_ptr {
    ( $mutex_arc:expr ) => {
        $mutex_arc
            .lock()
            .map_err(|_| FlameError::Internal("mutex ptr".to_string()))
    };
}

lazy_static! {
    static ref INSTANCE: Arc<FrontendClient> = Arc::new(FrontendClient {
        client_pool: Arc::new(Mutex::new(HashMap::new()))
    });
}

#[derive(Clone, Debug)]
pub struct FrontendClient {
    client_pool: Arc<Mutex<HashMap<String, FlameClient>>>,
}

pub async fn connect(addr: &str) -> Result<(), FlameError> {
    let client = FlameFrontendClient::connect(addr.to_string())
        .await
        .map_err(|_e| FlameError::Network("tonic connection".to_string()))?;

    let mut cs = lock_ptr!(INSTANCE.client_pool)?;
    cs.insert(FLAME_CLIENT_NAME.to_string(), client);

    Ok(())
}

fn get_client() -> Result<FlameClient, FlameError> {
    let cs = lock_ptr!(INSTANCE.client_pool)?;
    let client = cs
        .get(FLAME_CLIENT_NAME)
        .ok_or(FlameError::Internal("no flame client".to_string()))?;

    Ok(client.clone())
}

#[derive(Error, Debug)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum SessionState {
    Open = 0,
    Closed = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
pub enum TaskState {
    Pending = 0,
    Running = 1,
    Succeed = 2,
    Failed = 3,
}

pub struct SessionAttributes {
    pub application: String,
    pub slots: i32,
}

pub struct Session {
    pub id: SessionID,

    pub state: SessionState,
}

pub struct Task {
    pub id: TaskID,
    pub ssn_id: SessionID,

    pub state: TaskState,

    pub input: Option<TaskInput>,
    pub output: Option<TaskOutput>,
}

impl Task {
    pub fn is_completed(&self) -> bool {
        self.state == Succeed || self.state == Failed
    }
}

impl Session {
    pub async fn new(attrs: &SessionAttributes) -> Result<Self, FlameError> {
        let mut client = get_client()?;
        let create_ssn_req = CreateSessionRequest {
            session: Some(SessionSpec {
                application: attrs.application.clone(),
                slots: attrs.slots,
            }),
        };

        let ssn = client.create_session(create_ssn_req).await?;
        let ssn = ssn.into_inner();

        Ok(Session::from(&ssn))
    }

    pub async fn create_task(&self, input: TaskInput) -> Result<Task, FlameError> {
        let mut client = get_client()?;
        let create_task_req = CreateTaskRequest {
            task: Some(TaskSpec {
                session_id: self.id.clone(),
                input: Some(input.to_vec()),
                output: None,
            }),
        };
        let task = client.create_task(create_task_req).await?;

        let task = task.into_inner();
        Ok(Task::from(&task))
    }

    pub async fn get_task(&self, id: TaskID) -> Result<Task, FlameError> {
        let mut client = get_client()?;
        let get_task_req = GetTaskRequest {
            session_id: self.id.clone(),
            task_id: id.clone(),
        };
        let task = client.get_task(get_task_req).await?;

        let task = task.into_inner();
        Ok(Task::from(&task))
    }

    pub async fn close(&self) -> Result<(), FlameError> {
        let mut client = get_client()?;
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
            id: metadata.id.clone(),
            ssn_id: spec.session_id.clone(),
            input: spec.input.map(TaskInput::from),
            output: spec.output.map(TaskOutput::from),
            state: TaskState::from_i32(status.state).unwrap_or(TaskState::default()),
        }
    }
}

impl From<&rpc::Session> for Session {
    fn from(ssn: &rpc::Session) -> Self {
        let metadata = ssn.metadata.clone().unwrap();
        let status = ssn.status.clone().unwrap();
        Session {
            id: metadata.id.clone(),
            state: SessionState::from_i32(status.state).unwrap_or(SessionState::default()),
        }
    }
}
