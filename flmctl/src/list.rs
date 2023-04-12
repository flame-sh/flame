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

use chrono::{DateTime, NaiveDateTime, Utc};
use tonic::Status;

use rpc::flame::frontend_client::FrontendClient;

use rpc::flame::{ListSessionRequest, SessionState};

use crate::FLAME_SERVER;

pub async fn run() -> Result<(), Box<dyn Error>> {
    let addr = env::var(FLAME_SERVER)?;

    log::debug!("Flame server is: {}", addr);
    let mut client = FrontendClient::connect(addr).await?;

    let ssn_list = client.list_session(ListSessionRequest {}).await?;

    println!(
        "{:<10}{:<10}{:<15}{:<10}{:<10}{:<10}{:<10}{:<10}{:<10}",
        "ID", "State", "App", "Slots", "Pending", "Running", "Succeed", "Failed", "Created"
    );

    for ssn in &(ssn_list.into_inner().sessions) {
        let meta = ssn.metadata.clone().ok_or(Status::data_loss("no meta"))?;
        let spec = ssn.spec.clone().ok_or(Status::data_loss("no spec"))?;
        let status = ssn.status.clone().ok_or(Status::data_loss("no status"))?;
        let state = SessionState::from_i32(status.state).ok_or(Status::data_loss("no state"))?;

        let naivedatetime_utc = NaiveDateTime::from_timestamp_millis(status.creation_time * 1000)
            .ok_or(Status::data_loss("no creation_time"))?;
        let created = DateTime::<Utc>::from_utc(naivedatetime_utc, Utc);

        println!(
            "{:<10}{:<10}{:<15}{:<10}{:<10}{:<10}{:<10}{:<10}{:<10}",
            meta.id,
            state.as_str_name(),
            spec.application,
            spec.slots,
            status.pending,
            status.running,
            status.succeed,
            status.failed,
            created.format("%T")
        );
    }

    Ok(())
}
