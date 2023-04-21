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

use clap::Parser;

use flame_client as flame;
use flame_client::{Session, SessionAttributes, TaskInput};

#[derive(Parser)]
#[command(name = "pi")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Pi Example", long_about = None)]
struct Cli {
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

    let app = cli.app.unwrap_or(DEFAULT_APP.to_string());
    let slots = cli.slots.unwrap_or(DEFAULT_SLOTS);
    let task_input_str = cli.task_input.unwrap_or(DEFAULT_TASK_INPUT).to_string();
    let task_input = task_input_str
        .clone()
        .parse::<i32>()
        .unwrap_or(DEFAULT_TASK_INPUT);
    let task_num = cli.task_num.unwrap_or(DEFAULT_TASK_NUM);

    flame::connect("http://127.0.0.1:8080").await?;
    let ssn = Session::new(&SessionAttributes {
        application: app,
        slots,
    })
    .await?;

    let mut task_ids = vec![];
    for _ in 0..task_num {
        let task_input = task_input_str.as_bytes().to_vec();
        let task = ssn.create_task(TaskInput::from(task_input)).await?;
        task_ids.push(task.id.clone());
    }

    loop {
        let mut succeed = 0;
        let mut area: i64 = 0;

        for id in &task_ids {
            let task = ssn.get_task(id.to_string()).await?;
            if task.is_completed() {
                succeed += 1;
                if let Some(output) = task.output {
                    let output_str = String::from_utf8(output.to_vec())?;
                    area += output_str.trim().parse::<i64>().unwrap();
                }
            }
        }

        // If all tasks finished, exit.
        if task_ids.len() == succeed {
            let pi = 4_f64 * area as f64 / ((task_num as f64) * (task_input as f64));
            println!("pi = 4*({}/{}) = {}", area, task_num * task_input, pi);
            break;
        }

        thread::sleep(time::Duration::from_millis(300));
    }

    ssn.close().await?;

    Ok(())
}
