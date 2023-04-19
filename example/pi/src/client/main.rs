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

use std::error::Error;
use std::{thread, time};

use chrono::Local;
use clap::Parser;
use common::FlameContext;
use tonic::Status;

use rpc::flame::frontend_client::FrontendClient;

use rpc::flame::{
    CloseSessionRequest, CreateSessionRequest, CreateTaskRequest, GetTaskRequest, SessionSpec,
    TaskSpec, TaskState,
};

#[derive(Parser)]
#[command(name = "pi")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Pi Example", long_about = None)]
struct Cli {
    #[arg(long)]
    flame_conf: Option<String>,
    #[arg(short, long)]
    app: Option<String>,
    #[arg(short, long)]
    slots: Option<i32>,
    #[arg(long)]
    task_num: Option<i32>,
    #[arg(long)]
    task_input: Option<i32>,
}

const DEFAULT_APP: &str = "pi";
const DEFAULT_SLOTS: i32 = 1;
const DEFAULT_TASK_NUM: i32 = 100;
const DEFAULT_TASK_INPUT: i32 = 10000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();

    let ctx = FlameContext::from_file(cli.flame_conf)?;
    let mut client = FrontendClient::connect(ctx.endpoint).await?;

    let app = cli.app.unwrap_or(DEFAULT_APP.to_string());
    let slots = cli.slots.unwrap_or(DEFAULT_SLOTS);
    let task_input_str = cli.task_input.unwrap_or(DEFAULT_TASK_INPUT).to_string();
    let task_input = task_input_str.parse::<i32>().unwrap_or(DEFAULT_TASK_INPUT);
    let task_num = cli.task_num.unwrap_or(DEFAULT_TASK_NUM);

    let create_ssn_req = CreateSessionRequest {
        session: Some(SessionSpec {
            application: app.clone(),
            slots,
        }),
    };

    let ssn_creation_start_time = Local::now();
    let ssn = client.create_session(create_ssn_req).await?;
    let ssn_creation_end_time = Local::now();

    let ssn_meta = ssn
        .into_inner()
        .metadata
        .clone()
        .ok_or(Status::data_loss("no session meta"))?;

    let mut task_ids = vec![];

    let tasks_creations_start_time = Local::now();
    for _ in 0..cli.task_num.unwrap_or(DEFAULT_TASK_NUM) {
        let create_task_req = CreateTaskRequest {
            task: Some(TaskSpec {
                session_id: ssn_meta.id.clone(),
                input: Some(task_input_str.clone()),
                output: None,
            }),
        };
        let task = client.create_task(create_task_req).await?;
        let task_meta = task
            .into_inner()
            .metadata
            .clone()
            .ok_or(Status::data_loss("no task meta"))?;

        task_ids.push(task_meta.id.clone());
    }
    let tasks_creation_end_time = Local::now();

    let ssn_creation_time =
        ssn_creation_end_time.timestamp_millis() - ssn_creation_start_time.timestamp_millis();
    let tasks_creation_time =
        tasks_creation_end_time.timestamp_millis() - tasks_creations_start_time.timestamp_millis();

    println!(
        "Create session in <{} ms>, and create <{}> tasks in <{} ms>.\n",
        ssn_creation_time,
        task_ids.len(),
        tasks_creation_time
    );
    println!("Waiting for <{}> tasks to complete:", task_ids.len());

    let mut area = 0;

    for _ in 0..1000 {
        let mut succeed = 0;

        for id in &task_ids {
            let get_task_req = GetTaskRequest {
                session_id: ssn_meta.id.clone(),
                task_id: id.clone(),
            };
            let task = client.get_task(get_task_req).await?.into_inner();

            let task_status = task
                .clone()
                .status
                .clone()
                .ok_or(Status::data_loss("no task status"))?;

            let state =
                TaskState::from_i32(task_status.state).ok_or(Status::data_loss("no task state"))?;

            if TaskState::TaskSucceed == state {
                succeed += 1;

                let spec = task.clone().spec.ok_or(Status::data_loss("no task spec"))?;
                let output = spec
                    .output
                    .clone()
                    .ok_or(Status::data_loss("no task output"))?;
                area += output.parse::<i32>().unwrap();
            }
        }

        // If all tasks finished, exit.
        if task_ids.len() == succeed {
            println!("pi = 4*({}/{})", area, task_num * task_input);
            break;
        }

        thread::sleep(time::Duration::from_secs(1));
    }

    let close_ssn_req = CloseSessionRequest {
        session_id: ssn_meta.id.clone(),
    };
    client.close_session(close_ssn_req).await?;

    Ok(())
}
