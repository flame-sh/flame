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
    application: &String,
) -> Result<(), Box<dyn Error>> {
    let application = conn.get_application(application).await?;
    println!("Name: {}", application.name);
    println!("Shim: {}", application.attributes.shim);
    println!("URL: {}", application.attributes.url.unwrap_or_default());
    println!(
        "Command: {}",
        application.attributes.command.unwrap_or_default()
    );
    println!("Arguments: {:?}", application.attributes.arguments);
    println!("Environments: {:?}", application.attributes.environments);
    println!(
        "Working Directory: {:?}",
        application.attributes.working_directory
    );
    Ok(())
}
