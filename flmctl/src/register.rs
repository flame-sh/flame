/*
Copyright 2025 The Flame Authors.
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

use std::{collections::HashMap, fs, path::Path};

use flame_rs as flame;
use flame_rs::apis::Shim;
use flame_rs::{
    apis::{FlameContext, FlameError},
    client::ApplicationAttributes,
};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetadataYaml {
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpecYaml {
    pub shim: Option<String>,
    pub url: Option<String>,
    pub command: Option<String>,
    pub arguments: Option<Vec<String>>,
    pub environments: Option<HashMap<String, String>>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApplicationYaml {
    pub metadata: MetadataYaml,
    pub spec: SpecYaml,
}

pub async fn run(ctx: &FlameContext, path: &String) -> Result<(), FlameError> {
    if !Path::new(&path).is_file() {
        return Err(FlameError::InvalidConfig(format!(
            "<{path}> is not a file"
        )));
    }

    let contents =
        fs::read_to_string(path.clone()).map_err(|e| FlameError::Internal(e.to_string()))?;
    let app: ApplicationYaml =
        serde_yaml::from_str(&contents).map_err(|e| FlameError::Internal(e.to_string()))?;

    let app_attr = ApplicationAttributes::try_from(&app)?;

    let conn = flame::client::connect(&ctx.endpoint).await?;

    conn.register_application(app.metadata.name, app_attr)
        .await?;
    Ok(())
}

impl TryFrom<&ApplicationYaml> for ApplicationAttributes {
    type Error = FlameError;

    fn try_from(yaml: &ApplicationYaml) -> Result<Self, Self::Error> {
        let shim = match yaml
            .spec
            .shim
            .clone()
            .unwrap_or(String::from("grpc"))
            .to_lowercase()
            .as_str()
        {
            "grpc" => Ok(Shim::Grpc),
            _ => Err(FlameError::InvalidConfig("unsupported shim".to_string())),
        }?;

        Ok(Self {
            shim,
            url: yaml.spec.url.clone(),
            command: yaml.spec.command.clone(),
            arguments: yaml.spec.arguments.clone().unwrap_or_default(),
            environments: yaml.spec.environments.clone().unwrap_or_default(),
            working_directory: yaml.spec.working_directory.clone(),
        })
    }
}
