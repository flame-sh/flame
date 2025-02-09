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

use flame_rs as flame;
use flame_rs::{apis::FlameContext, client::SessionAttributes};

pub async fn run(ctx: &FlameContext, app: &str, slots: &i32) -> Result<(), Box<dyn Error>> {
    let conn = flame::client::connect(&ctx.endpoint).await?;
    let attr = SessionAttributes {
        application: app.to_owned(),
        slots: *slots,
        common_data: None,
    };

    let ssn = conn.create_session(&attr).await?;

    println!("Session <{}> was created.", ssn.id);

    Ok(())
}
