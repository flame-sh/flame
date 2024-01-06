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

use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::apis::Application;
use crate::FlameError;

const DEFAULT_FLAME_CONF: &str = "flame-conf.yaml";
const DEFAULT_CONTEXT_NAME: &str = "flame";
const DEFAULT_FLAME_ENDPOINT: &str = "http://127.0.0.1:8080";
const DEFAULT_SLOT: &str = "cpu=1,mem=2g";
const DEFAULT_POLICY: &str = "proportion";
const DEFAULT_STORAGE: &str = "sqlite://flame.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameContext {
    pub name: String,
    pub endpoint: String,
    pub slot: String,
    pub policy: String,
    pub storage: String,
    pub applications: Vec<Application>,
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
            slot: DEFAULT_SLOT.to_string(),
            policy: DEFAULT_POLICY.to_string(),
            storage: DEFAULT_STORAGE.to_string(),
            applications: vec![Application::default()],
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

        let contents = fs::read_to_string(fp.clone())
            .map_err(|_| FlameError::Internal("can not read flame-conf".to_string()))?;
        let ctx: FlameContext = serde_yaml::from_str(&contents)
            .map_err(|_| FlameError::Internal("flame-conf".to_string()))?;

        log::debug!("Load FrameContext from <{}>: {}", fp, ctx);

        if ctx.applications.is_empty() {
            return Err(FlameError::InvalidConfig("no application".to_string()));
        }

        Ok(ctx)
    }

    pub fn get_application(&self, n: &String) -> Option<Application> {
        let mut application = None;

        for app in &self.applications {
            if n == &app.name {
                application = Some(app.clone());
                break;
            }
        }

        application
    }
}
