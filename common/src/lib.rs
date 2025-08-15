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

pub mod apis;
pub mod ctx;
pub mod ptr;
pub mod trace;

use std::collections::HashMap;
use std::io::Write;
use std::process;

use chrono::Local;
use thiserror::Error;
use tonic::Status;

use crate::apis::{ApplicationAttributes, Shim};

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

    #[error("'{0}' is not initialized")]
    Uninitialized(String),

    #[error("'{0}'")]
    InvalidState(String),

    #[error("'{0}'")]
    Storage(String),
}

impl From<FlameError> for Status {
    fn from(value: FlameError) -> Self {
        match value {
            FlameError::NotFound(s) => Status::not_found(s),
            FlameError::Internal(s) => Status::internal(s),
            _ => Status::unknown("unknown"),
        }
    }
}

impl From<Status> for FlameError {
    fn from(value: Status) -> Self {
        FlameError::Network(value.code().to_string())
    }
}

#[macro_export]
macro_rules! lock_ptr {
    ( $mutex_arc:expr ) => {
        $mutex_arc
            .lock()
            .map_err(|_| FlameError::Internal("mutex ptr".to_string()))
    };
}

#[macro_export]
macro_rules! lock_async_ptr {
    ( $mutex_arc:expr ) => {
        $mutex_arc
            .lock()
            .await
            .map_err(|_| FlameError::Internal("mutex ptr".to_string()))
    };
}

// #[macro_export]
// macro_rules! lock_async_ptr {
//     ( $mutex_arc:expr ) => {
//         $mutex_arc
//             .lock()
//             .await
//             .map_err(|_| FlameError::Internal("async mutex ptr".to_string()))
//     };
// }

// #[macro_export]
// macro_rules! lock_cond_ptr {
//     ( $mutex_arc:expr ) => {
//         $mutex_arc
//             .ptr
//             .lock()
//             .map_err(|_| FlameError::Internal("cond ptr".to_string()))
//     };
// }
//
// #[derive(Clone, Debug, Copy, ::prost::Enumeration, Serialize, Deserialize)]
// pub enum Shim {
//     Log = 0,
//     Stdio = 1,
//     Rpc = 2,
//     Rest = 3,
// }
//
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Application {
//     pub name: String,
//     pub shim: Shim,
//     pub command_line: String,
//     pub working_directory: String,
// }
//
// impl Default for Application {
//     fn default() -> Self {
//         Application {
//             name: "flmexec".to_string(),
//             shim: Shim::Log,
//             command_line: "/usr/bin/flmexec".to_string(),
//             working_directory: "/tmp".to_string(),
//         }
//     }
// }

pub fn init_logger() {
    // Initialize env_logger with a custom format
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let now = Local::now();
            let pid = process::id(); // Get the current process ID
            writeln!(
                buf,
                "{} [{}] {} [{}]: {}",
                record.level(),
                now.format("%Y-%m-%d %H:%M:%S%.6f"),
                pid,
                record.target(),
                record.args()
            )
        })
        .init();
}

pub fn default_applications() -> HashMap<String, ApplicationAttributes> {
    HashMap::from([
        (
            "flmexec".to_string(),
            ApplicationAttributes {
                shim: Shim::Grpc,
                command: Some("/usr/local/flame/bin/flmexec-service".to_string()),
                ..ApplicationAttributes::default()
            },
        ),
        (
            "flmping".to_string(),
            ApplicationAttributes {
                shim: Shim::Grpc,
                command: Some("/usr/local/flame/bin/flmping-service".to_string()),
                ..ApplicationAttributes::default()
            },
        ),
        (
            "flmtest".to_string(),
            ApplicationAttributes {
                shim: Shim::Log,
                ..ApplicationAttributes::default()
            },
        ),
    ])
}
