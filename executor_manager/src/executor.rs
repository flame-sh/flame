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

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::states;
use common::{FlameContext, FlameError};
use rpc::flame::frontend_client::FrontendClient;

#[derive(Clone, Copy, Debug)]
pub enum ExecutorState {
    Initialized = 0,
    Idle = 1,
    Bound = 2,
    Running = 3,
    Unknown = 4,
}

#[derive(Clone, Debug)]
pub struct Application {
    pub name: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub environments: Vec<String>,
    pub working_directory: String,
}

impl From<&common::Application> for Application {
    fn from(app: &common::Application) -> Self {
        Application {
            name: app.name.to_string(),
            command: app.command_line.to_string(),
            arguments: vec![],
            environments: vec![],
            working_directory: app.working_directory.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Task {
    id: String,
    ssn_id: String,
    input: String,
}

#[derive(Clone, Debug)]
pub struct Executor {
    pub id: String,
    pub slots: i32,
    pub application: Application,
    pub task: Option<Task>,

    pub start_time: DateTime<Utc>,
    pub state: ExecutorState,
}

impl Executor {
    pub fn from_context(
        ctx: &FlameContext,
        appname: Option<String>,
        slot: Option<i32>,
    ) -> Result<Self, FlameError> {
        let application: Result<Application, FlameError> = match appname {
            Some(n) => {
                if let Some(app) = ctx.applications.iter().find(|&s| s.name == n) {
                    Ok(Application::from(app))
                } else {
                    return Err(FlameError::InvalidConfig(n));
                }
            }
            None => Err(FlameError::InvalidConfig("no application name".to_string())),
        };

        let mut exec = Executor {
            id: Uuid::new_v4().to_string(),
            slots: slot.unwrap_or(1),
            application: application?,
            task: None,
            start_time: Utc::now(),
            state: ExecutorState::Initialized,
        };

        Ok(exec)
    }

    pub async fn run<T>(
        &self,
        ctx: &FlameContext,
        client: &mut FrontendClient<T>,
    ) -> Result<(), FlameError> {
        let state = states::get_state(self)?;
        state.execute(ctx, client)
    }
}

pub async fn run(
    ctx: &FlameContext,
    appname: Option<String>,
    slot: Option<i32>,
) -> Result<(), FlameError> {
    let exec = Executor::from_context(ctx, appname, slot)?;
    let mut client = FrontendClient::connect(ctx.endpoint.clone())
        .await
        .map_err(|e| FlameError::Network("tonic connection".to_string()))?;

    loop {
        let res = exec.run(ctx, &mut client).await;
        if let Err(e) = res {
            log::error!("Failed to execute: {}", e);
        }
    }
}
