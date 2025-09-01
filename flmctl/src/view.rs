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

use std::error::Error;

use serde_json::Value;

use flame_rs::apis::{FlameContext, FlameError};
use flame_rs::client;

pub async fn run(
    ctx: &FlameContext,
    session: &Option<String>,
    application: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let conn = client::connect(&ctx.endpoint).await?;
    match (session, application) {
        (Some(session), None) => view_session(conn, session).await,
        (None, Some(application)) => view_application(conn, application).await,
        _ => Err(Box::new(FlameError::InvalidConfig(
            "unsupported parameters".to_string(),
        ))),
    }
}

async fn view_session(_: client::Connection, _: &String) -> Result<(), Box<dyn Error>> {
    todo!()
}

async fn view_application(
    conn: client::Connection,
    application: &str,
) -> Result<(), Box<dyn Error>> {
    let application = conn.get_application(application).await?;
    println!("{:<15}{}", "Name:", application.name);
    println!(
        "{:<15}{}",
        "Description:",
        application.attributes.description.unwrap_or_default()
    );
    println!("{:<15}{}", "Shim:", application.attributes.shim);
    println!(
        "{:<15}{}",
        "Image:",
        application.attributes.image.unwrap_or_default()
    );
    println!("{:<15}", "Labels:");
    for label in application.attributes.labels {
        println!("\t{label}");
    }
    println!(
        "{:<15}{}",
        "Command:",
        application.attributes.command.unwrap_or_default()
    );
    println!("{:<15}", "Arguments:");
    for argument in application.attributes.arguments {
        println!("\t{argument}");
    }
    println!("{:<15}", "Environments:");
    for (key, value) in application.attributes.environments {
        println!("\t{key}: {value}");
    }
    println!(
        "{:<15}{}",
        "WorkingDir:",
        application.attributes.working_directory.unwrap_or_default()
    );
    println!(
        "{:<15}{}",
        "Max Instances:",
        application.attributes.max_instances.unwrap_or_default()
    );
    println!(
        "{:<15}{}",
        "Delay Release:",
        application.attributes.delay_release.unwrap_or_default()
    );

    println!("{:<15}", "Schema:");

    if let Some(schema) = application.attributes.schema {
        let input_type = get_type(schema.input)?;
        let output_type = get_type(schema.output)?;
        let common_data_type = get_type(schema.common_data)?;

        println!("  Input: {input_type}");
        println!("  Output: {output_type}");
        println!("  Common Data: {common_data_type}");
    }
    Ok(())
}

fn get_type(schema: Option<String>) -> Result<String, FlameError> {
    match schema {
        Some(schema) => {
            let value = serde_json::from_str::<Value>(&schema)
                .map_err(|e| FlameError::InvalidConfig(e.to_string()))?;
            let schema_type = value.get("type").ok_or(FlameError::InvalidConfig(
                "schema type is missed".to_string(),
            ))?;
            Ok(schema_type.to_string())
        }
        None => Ok("-".to_string()),
    }
}
