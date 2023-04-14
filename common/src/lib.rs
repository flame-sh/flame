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

pub mod ptr;

// pub use ptr::{CondPtr, MutexPtr};

use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tonic::Status;

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
            .map_err(|_| FlameError::Internal("mutex".to_string()))
    };
}

#[macro_export]
macro_rules! lock_cond_ptr {
    ( $mutex_arc:expr ) => {
        $mutex_arc
            .ptr
            .lock()
            .map_err(|_| FlameError::Internal("mutex".to_string()))
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub command_line: String,
    pub working_directory: String,
}

impl Default for Application {
    fn default() -> Self {
        Application {
            name: "flmexec".to_string(),
            command_line: "/usr/bin/flmexec".to_string(),
            working_directory: "/tmp".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameContext {
    pub name: String,
    pub endpoint: String,
    pub slot: String,
    pub policy: String,
    pub storage: String,
    pub applications: Vec<Application>,
}

impl Default for FlameContext {
    fn default() -> Self {
        FlameContext {
            name: "flame".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            slot: "cpu=1,mem=1g".to_string(),
            policy: "priority".to_string(),
            storage: "mem".to_string(),
            applications: vec![Application::default()],
        }
    }
}

const DEFAULT_FLAME_CONF: &str = "flame-conf.yaml";

impl FlameContext {
    pub fn from_file(fp: Option<String>) -> Result<Self, FlameError> {
        let fp = fp.unwrap_or(DEFAULT_FLAME_CONF.to_string());

        let ctx: FlameContext =
            confy::load_path(fp).map_err(|_| FlameError::Internal("flame-conf".to_string()))?;

        if ctx.applications.len() == 0 {
            return Err(FlameError::InvalidConfig("no application".to_string()));
        }

        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use crate::ptr::CondPtr;
    use std::{thread, time};

    #[test]
    fn test_ptr() {
        let pair = CondPtr::new(false);
        let pair2 = pair.clone();

        thread::spawn(move || {
            let delay = time::Duration::from_millis(3000);
            thread::sleep(delay);
            pair.modify(|p| {
                *p = true;
                println!("Update condition: {}", *p);
                Ok(())
            })
            .unwrap();
        });

        pair2
            .wait_while(|pending| {
                println!("Waiting for condition: {}", *pending);
                *pending
            })
            .unwrap();
    }
}
