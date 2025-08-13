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

use chrono::{DateTime, TimeZone, Utc};
use futures::TryFutureExt;
use stdng::{logs::TraceFn, trace_fn};
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use tonic::transport::Endpoint;
use tonic::Request;

use self::rpc::frontend_client::FrontendClient as FlameFrontendClient;
use self::rpc::{
    ApplicationSpec, CloseSessionRequest, CreateSessionRequest, CreateTaskRequest, Environment,
    GetApplicationRequest, GetTaskRequest, ListApplicationRequest, ListSessionRequest,
    RegisterApplicationRequest, SessionSpec, TaskSpec, WatchTaskRequest,
};
use crate::apis::flame as rpc;
use crate::apis::Shim;
use crate::apis::{
    ApplicationID, ApplicationState, CommonData, FlameError, SessionID, SessionState, TaskID,
    TaskInput, TaskOutput, TaskState,
};
use crate::lock_ptr;

type FlameClient = FlameFrontendClient<Channel>;

pub async fn connect(addr: &str) -> Result<Connection, FlameError> {
    let endpoint = Endpoint::from_shared(addr.to_string())
        .map_err(|_| FlameError::InvalidConfig("invalid address".to_string()))?;

    let channel = endpoint
        .connect()
        .await
        .map_err(|_| FlameError::InvalidConfig("failed to connect".to_string()))?;

    Ok(Connection { channel })
}

#[derive(Clone)]
pub struct Connection {
    pub(crate) channel: Channel,
}

#[derive(Clone)]
pub struct SessionAttributes {
    pub application: String,
    pub slots: i32,
    pub common_data: Option<CommonData>,
}

#[derive(Clone)]
pub struct ApplicationAttributes {
    pub shim: Shim,

    pub url: Option<String>,
    pub command: Option<String>,
    pub arguments: Vec<String>,
    pub environments: HashMap<String, String>,
    pub working_directory: Option<String>,
}

#[derive(Clone)]
pub struct Application {
    pub name: ApplicationID,

    pub attributes: ApplicationAttributes,

    pub state: ApplicationState,
    pub creation_time: DateTime<Utc>,
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
                common_data: attrs.common_data.clone().map(CommonData::into),
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

    pub async fn register_application(
        &self,
        name: String,
        app: ApplicationAttributes,
    ) -> Result<(), FlameError> {
        let mut client = FlameClient::new(self.channel.clone());

        let req = RegisterApplicationRequest {
            name,
            application: Some(ApplicationSpec::from(app)),
        };

        let res = client
            .register_application(Request::new(req))
            .await?
            .into_inner();

        if res.return_code < 0 {
            Err(FlameError::Network(res.message.unwrap_or_default()))
        } else {
            Ok(())
        }
    }

    pub async fn list_application(&self) -> Result<Vec<Application>, FlameError> {
        let mut client = FlameClient::new(self.channel.clone());
        let app_list = client.list_application(ListApplicationRequest {}).await?;

        Ok(app_list
            .into_inner()
            .applications
            .iter()
            .map(Application::from)
            .collect())
    }

    pub async fn get_application(&self, name: &str) -> Result<Application, FlameError> {
        let mut client = FlameClient::new(self.channel.clone());
        let app = client
            .get_application(GetApplicationRequest {
                name: name.to_string(),
            })
            .await?;
        Ok(Application::from(&app.into_inner()))
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

impl From<&rpc::Task> for Task {
    fn from(task: &rpc::Task) -> Self {
        let metadata = task.metadata.clone().unwrap();
        let spec = task.spec.clone().unwrap();
        let status = task.status.unwrap();
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
        let status = ssn.status.unwrap();
        let spec = ssn.spec.clone().unwrap();

        let naivedatetime_utc =
            DateTime::from_timestamp_millis(status.creation_time * 1000).unwrap();
        let creation_time = Utc.from_utc_datetime(&naivedatetime_utc.naive_utc());

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

impl From<&rpc::Application> for Application {
    fn from(app: &rpc::Application) -> Self {
        let metadata = app.metadata.clone().unwrap();
        let spec = app.spec.clone().unwrap();
        let status = app.status.unwrap();

        let naivedatetime_utc =
            DateTime::from_timestamp_millis(status.creation_time * 1000).unwrap();
        let creation_time = Utc.from_utc_datetime(&naivedatetime_utc.naive_utc());

        Self {
            name: metadata.name,
            attributes: ApplicationAttributes::from(spec),
            state: ApplicationState::from(status.state()),
            creation_time,
        }
    }
}

impl From<ApplicationAttributes> for ApplicationSpec {
    fn from(app: ApplicationAttributes) -> Self {
        Self {
            shim: app.shim.into(),
            url: app.url.clone(),
            command: app.command.clone(),
            arguments: app.arguments.clone(),
            environments: app
                .environments
                .clone()
                .into_iter()
                .map(|(key, value)| Environment { name: key, value })
                .collect(),
            working_directory: app.working_directory.clone(),
        }
    }
}

impl From<ApplicationSpec> for ApplicationAttributes {
    fn from(app: ApplicationSpec) -> Self {
        Self {
            shim: app.shim().into(),
            url: app.url.clone(),
            command: app.command.clone(),
            arguments: app.arguments.clone(),
            environments: app
                .environments
                .clone()
                .into_iter()
                .map(|env| (env.name, env.value))
                .collect(),
            working_directory: app.working_directory.clone(),
        }
    }
}
