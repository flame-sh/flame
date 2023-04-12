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

use clap::Parser;
use tonic::Status;

use rpc::flame::frontend_client::FrontendClient;

use rpc::flame::{
    CreateSessionRequest, CreateTaskRequest,  SessionSpec,
    TaskSpec,
};

#[derive(Parser)]
#[command(name = "fping")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Ping", long_about = None)]
struct Cli {
    #[arg(short, long)]
    app: String,
    #[arg(short, long)]
    slots: i32,
    #[arg(short, long)]
    task_num: i32,
}

const APISERVER: &str = "FLAME_APISERVER";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();

    let addr = env::var(APISERVER)?;
    let mut client = FrontendClient::connect(addr).await?;

    let create_ssn_req = CreateSessionRequest {
        session: Some(SessionSpec {
            application: cli.app.clone(),
            slots: cli.slots,
        }),
    };

    let ssn = client.create_session(create_ssn_req).await?;
    let ssn_meta = ssn
        .into_inner()
        .metadata
        .clone()
        .ok_or(Status::data_loss("no meta"))?;

    let task_ids = vec![];

    for _ in 1..cli.task_num {
        let create_task_req = CreateTaskRequest {
            task: Some(TaskSpec {
                session_id: ssn_meta.id.clone(),
                input: None,
                output: None,
            }),
        };
        let task = client.create_task(create_task_req).await?;
    }

    for _ in 1..cli.task_num {
        let get_task_req = GetTaskRequest {
            task: Some(TaskSpec {
                session_id: ssn_meta.id.clone(),
                input: None,
                output: None,
            }),
        };
        client.get_task(get_task_req).await?;
    }

    Ok(())
}
