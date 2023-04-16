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

use std::sync::Arc;
use tonic::transport::Server;

use rpc::flame::backend_server::BackendServer;
use rpc::flame::frontend_server::FrontendServer;
use rpc::flame::{
    Application, Metadata, Session, SessionSpec, SessionState, SessionStatus, Task, TaskSpec,
    TaskState, TaskStatus,
};

use crate::storage::Storage;
use crate::{model, storage, FlameError};

mod backend;
mod frontend;

pub struct Flame {
    storage: Arc<Storage>,
}

pub async fn run() -> Result<(), FlameError> {
    let address = "[::1]:8080".parse().unwrap();
    let frontend_service = Flame {
        storage: storage::instance(),
    };

    let backend_service = Flame {
        storage: storage::instance(),
    };

    Server::builder()
        // TODO(k82cn): separate frontend & backend concurrent limit.
        .concurrency_limit_per_connection(6000)
        .add_service(FrontendServer::new(frontend_service))
        .add_service(BackendServer::new(backend_service))
        .serve(address)
        .await
        .map_err(|e| FlameError::Network(e.to_string()))?;

    Ok(())
}

impl From<model::TaskState> for TaskState {
    fn from(state: model::TaskState) -> Self {
        match state {
            model::TaskState::Pending => TaskState::TaskPending,
            model::TaskState::Running => TaskState::TaskRunning,
            model::TaskState::Succeed => TaskState::TaskSucceed,
            model::TaskState::Failed => TaskState::TaskFailed,
        }
    }
}

impl From<&model::Task> for Task {
    fn from(task: &model::Task) -> Self {
        Task {
            metadata: Some(Metadata {
                id: task.id.to_string(),
                owner: Some(task.ssn_id.to_string()),
            }),
            spec: Some(TaskSpec {
                session_id: task.ssn_id.to_string(),
                input: task.input.clone(),
                output: task.output.clone(),
            }),
            status: Some(TaskStatus {
                state: TaskState::from(task.state) as i32,
                creation_time: task.creation_time.timestamp(),
                completion_time: match task.completion_time {
                    None => None,
                    Some(s) => Some(s.timestamp()),
                },
            }),
        }
    }
}

impl From<model::SessionState> for SessionState {
    fn from(state: model::SessionState) -> Self {
        match state {
            model::SessionState::Open => SessionState::SessionOpen,
            model::SessionState::Closed => SessionState::SessionClosed,
        }
    }
}

impl From<&model::Session> for Session {
    fn from(ssn: &model::Session) -> Self {
        let mut status = SessionStatus {
            state: SessionState::from(ssn.status.state) as i32,
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
                model::TaskState::Succeed => status.succeed = v.len() as i32,
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

impl From<Application> for model::Application {
    fn from(app: Application) -> Self {
        model::Application::from(&app)
    }
}

impl From<&Application> for model::Application {
    fn from(app: &Application) -> Self {
        model::Application {
            name: app.name.to_string(),
            command: app.command.to_string(),
            arguments: app.arguments.to_vec(),
            environments: app.environments.to_vec(),
            working_directory: app.working_directory.to_string(),
        }
    }
}
