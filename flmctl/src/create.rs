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

use std::env;
use std::error::Error;

use tonic::Status;

use common::FlameContext;
use rpc::flame::frontend_client::FrontendClient;
use rpc::flame::{CreateSessionRequest, SessionSpec};

pub async fn run(ctx: &FlameContext, app: &String, slots: &i32) -> Result<(), Box<dyn Error>> {
    let mut client = FrontendClient::connect(ctx.endpoint.clone()).await?;

    let req = CreateSessionRequest {
        session: Some(SessionSpec {
            application: app.clone(),
            slots: *slots,
        }),
    };
    let ssn = client.create_session(req).await?;

    let meta = ssn
        .into_inner()
        .metadata
        .ok_or(Status::data_loss("no session id"))?;

    println!("Session <{}> was created.", meta.id);

    Ok(())
}
