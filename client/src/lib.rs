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
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::TryFutureExt;
use lazy_static::lazy_static;
use prost::Enumeration;
use thiserror::Error;

use tokio_stream::StreamExt;

use tonic::transport::Channel;
use tonic::Status;

use self::rpc::frontend_client::FrontendClient as FlameFrontendClient;
use self::rpc::{
    CloseSessionRequest, CreateSessionRequest, CreateTaskRequest, GetTaskRequest, SessionSpec,
    TaskSpec, WatchTaskRequest,
};
use ::rpc::flame as rpc;

type FlameClient = FlameFrontendClient<Channel>;
type TaskID = String;
type SessionID = String;

type Message = Bytes;
pub type TaskInput = Message;
pub type TaskOutput = Message;

const FLAME_CLIENT_NAME: &str = "flame";

#[macro_export]
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
struct FrontendClient {
    client_pool: Arc<Mutex<HashMap<String, FlameClient>>>,
}

fn get_client() -> Result<FlameClient, FlameError> {
    let cs = lock_ptr!(INSTANCE.client_pool)?;
    let client = cs
        .get(FLAME_CLIENT_NAME)
        .ok_or(FlameError::Internal("no flame client".to_string()))?;

    Ok(client.clone())
}

pub async fn connect(addr: &str) -> Result<(), FlameError> {
    let client = FlameFrontendClient::connect(addr.to_string())
        .await
        .map_err(|_e| FlameError::Network("tonic connection".to_string()))?;

    let mut cs = lock_ptr!(INSTANCE.client_pool)?;
    cs.insert(FLAME_CLIENT_NAME.to_string(), client);

    Ok(())
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

#[derive(Clone)]
pub struct SessionAttributes {
    pub application: String,
    pub slots: i32,
}

#[derive(Clone)]
pub struct Session {
    pub id: SessionID,
    //
    // pub task_results: Arc<Mutex<HashMap<TaskID, Result<Task, FlameError>>>>,
    pub state: SessionState,
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
    fn on_task_updated(&mut self, task: Task);
    fn on_error(&mut self, e: FlameError);
}

impl Task {
    pub fn is_completed(&self) -> bool {
        self.state == TaskState::Succeed || self.state == TaskState::Failed
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

    pub async fn run_task(
        &self,
        input: TaskInput,
        informer_ptr: TaskInformerPtr,
    ) -> Result<(), FlameError> {
        self.create_task(input)
            .and_then(|task| self.watch_task(task.ssn_id, task.id, informer_ptr))
            .await
    }

    pub async fn watch_task(
        &self,
        session_id: SessionID,
        task_id: TaskID,
        informer_ptr: TaskInformerPtr,
    ) -> Result<(), FlameError> {
        let mut client = get_client()?;
        let watch_task_req = WatchTaskRequest {
            session_id,
            task_id,
        };
        let mut task_stream = client.watch_task(watch_task_req).await?.into_inner();
        while let Some(task) = task_stream.next().await {
            match task {
                Ok(t) => {
                    let mut informer = lock_ptr!(informer_ptr)?;
                    informer.on_task_updated(Task::from(&t));
                }
                Err(e) => {
                    let mut informer = lock_ptr!(informer_ptr)?;
                    informer.on_error(FlameError::from(e.clone()));
                }
            }
        }
        Ok(())
    }

    // pub fn run_task(
    //     &self,
    //     input: TaskInput,
    //     informer_ptr: TaskInformerPtr,
    // ) -> BoxFuture<'static, Result<(), FlameError>> {
    //     let rt_result = Runtime::new()
    //         .map_err(|_| FlameError::Internal("failed to start tokio runtime".to_string()));
    //
    //     if rt_result.is_err() {
    //         return Box::pin(WatchTaskFuture {
    //             task_result: Arc::new(Mutex::new(Err(rt_result.err().unwrap()))),
    //         });
    //     }
    //     let rt = rt_result.unwrap();
    //
    //     // Execute the future, blocking the current thread until completion
    //     let create_task_future = self.create_task(input);
    //     let task_result = rt.block_on(create_task_future);
    //
    //     if task_result.is_err() {
    //         return Box::pin(WatchTaskFuture {
    //             task_result: Arc::new(Mutex::new(Err(task_result.err().unwrap()))),
    //         });
    //     }
    //     let task = task_result.unwrap();
    //
    //     // let tasks_ptr = self.task_results.clone();
    //     let task_id = task.id.clone();
    //     let session_id = task.ssn_id.clone();
    //
    //     let task_result_ptr: TaskResultPtr = Arc::new(Mutex::new(Ok(task)));
    //     let task_result_ptr_future = task_result_ptr.clone();
    //
    //     let watch_task_stream = async move {
    //         let mut client = get_client()?;
    //         let watch_task_req = WatchTaskRequest {
    //             session_id,
    //             task_id: task_id.clone(),
    //         };
    //         let mut task_stream = client.watch_task(watch_task_req).await?.into_inner();
    //         while let Some(task_new) = task_stream.next().await {
    //             let mut task = lock_ptr!(task_result_ptr)?;
    //             match task_new {
    //                 Ok(t) => {
    //                     *task = Ok(Task::from(&t));
    //                     let mut informer = lock_ptr!(informer_ptr)?;
    //                     informer.on_task_updated(task.as_ref().unwrap().clone());
    //                 }
    //                 Err(e) => {
    //                     *task = Err(FlameError::from(e.clone()));
    //                     let mut informer = lock_ptr!(informer_ptr)?;
    //                     informer.on_error(FlameError::from(e.clone()));
    //                 }
    //             }
    //         }
    //
    //         Ok::<(), FlameError>(())
    //     };
    //
    //     tokio::spawn(watch_task_stream);
    //
    //     Box::pin(WatchTaskFuture {
    //         task_result: task_result_ptr_future,
    //     })
    // }

    pub async fn close(&self) -> Result<(), FlameError> {
        let mut client = get_client()?;
        let close_ssn_req = CloseSessionRequest {
            session_id: self.id.clone(),
        };

        client.close_session(close_ssn_req).await?;

        Ok(())
    }
}

struct WatchTaskFuture {
    task_result: Arc<Mutex<Result<Task, FlameError>>>,
}

impl Future for WatchTaskFuture {
    type Output = Result<(), FlameError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let task = lock_ptr!(self.task_result)?;
        match &*task {
            Ok(t) => {
                if t.is_completed() {
                    return Poll::Ready(Ok(()));
                }
            }
            Err(e) => {
                return Poll::Ready(Err(e.clone()));
            }
        }

        cx.waker().wake_by_ref();
        Poll::Pending
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
            state: TaskState::from_i32(status.state).unwrap_or(TaskState::default()),
        }
    }
}

impl From<&rpc::Session> for Session {
    fn from(ssn: &rpc::Session) -> Self {
        let metadata = ssn.metadata.clone().unwrap();
        let status = ssn.status.clone().unwrap();
        Session {
            id: metadata.id,
            state: SessionState::from_i32(status.state).unwrap_or(SessionState::default()),
        }
    }
}
