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

pub(crate) mod flame {
    tonic::include_proto!("flame");
}
use flame as rpc;

use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use bytes::Bytes;
use prost::Enumeration;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tonic::Status;

pub type TaskID = String;
pub type SessionID = String;
pub type ApplicationID = String;

type Message = Bytes;
pub type TaskInput = Message;
pub type TaskOutput = Message;
pub type CommonData = Message;

const DEFAULT_FLAME_CONF: &str = "flame-conf.yaml";
const DEFAULT_CONTEXT_NAME: &str = "flame";
const DEFAULT_FLAME_ENDPOINT: &str = "http://127.0.0.1:8080";

#[macro_export]
macro_rules! lock_ptr {
    ( $mutex_arc:expr ) => {
        $mutex_arc
            .lock()
            .map_err(|_| FlameError::Internal("mutex ptr".to_string()))
    };
}

#[macro_export]
macro_rules! new_ptr {
    ( $mutex_arc:expr ) => {
        Arc::new(Mutex::new($mutex_arc))
    };
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration, strum_macros::Display)]
pub enum ApplicationState {
    Enabled = 0,
    Disabled = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration, strum_macros::Display)]
pub enum Shim {
    Log = 0,
    Stdio = 1,
    Wasm = 2,
    Shell = 3,
    Grpc = 4,
}

impl From<Status> for FlameError {
    fn from(value: Status) -> Self {
        FlameError::Network(value.code().to_string())
    }
}

impl From<rpc::Shim> for Shim {
    fn from(shim: rpc::Shim) -> Self {
        match shim {
            rpc::Shim::Log => Shim::Log,
            rpc::Shim::Stdio => Shim::Stdio,
            rpc::Shim::Wasm => Shim::Wasm,
            rpc::Shim::Shell => Shim::Shell,
            rpc::Shim::Grpc => Shim::Grpc,
        }
    }
}

impl From<rpc::ApplicationState> for ApplicationState {
    fn from(s: rpc::ApplicationState) -> Self {
        match s {
            rpc::ApplicationState::Disabled => Self::Disabled,
            rpc::ApplicationState::Enabled => Self::Enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameContext {
    pub name: String,
    pub endpoint: String,
}

impl Display for FlameContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, endpoint: {}", self.name, self.endpoint)
    }
}

impl Default for FlameContext {
    fn default() -> Self {
        FlameContext {
            name: DEFAULT_CONTEXT_NAME.to_string(),
            endpoint: DEFAULT_FLAME_ENDPOINT.to_string(),
        }
    }
}

impl FlameContext {
    pub fn from_file(fp: Option<String>) -> Result<Self, FlameError> {
        let fp = match fp {
            None => {
                format!("{}/.flame/{}", env!("HOME", "."), DEFAULT_FLAME_CONF)
            }
            Some(path) => path,
        };

        if !Path::new(&fp).is_file() {
            return Err(FlameError::InvalidConfig(format!("<{}> is not a file", fp)));
        }

        let contents =
            fs::read_to_string(fp.clone()).map_err(|e| FlameError::Internal(e.to_string()))?;
        let ctx: FlameContext =
            serde_yaml::from_str(&contents).map_err(|e| FlameError::Internal(e.to_string()))?;

        log::debug!("Load FrameContext from <{}>: {}", fp, ctx);

        Ok(ctx)
    }
}
