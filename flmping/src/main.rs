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

use futures::future::try_join_all;
use std::error::Error;
use std::sync::{Arc, Mutex};

use chrono::Local;
use clap::Parser;
use indicatif::{HumanCount, ProgressBar, ProgressStyle};

use self::flame::FlameError;
use common::ctx::FlameContext;
use flame_client as flame;

#[derive(Parser)]
#[command(name = "flmping")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Ping", long_about = None)]
struct Cli {
    #[arg(long)]
    flame_conf: Option<String>,
    #[arg(short, long)]
    app: Option<String>,
    #[arg(short, long)]
    slots: Option<i32>,
    #[arg(short, long)]
    task_num: Option<i32>,
}

const DEFAULT_APP: &str = "flmexec";
const DEFAULT_SLOTS: i32 = 1;
const DEFAULT_TASK_NUM: i32 = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();

    let ctx = FlameContext::from_file(cli.flame_conf)?;

    let app = cli.app.unwrap_or(DEFAULT_APP.to_string());
    let slots = cli.slots.unwrap_or(DEFAULT_SLOTS);
    let task_num = cli.task_num.unwrap_or(DEFAULT_TASK_NUM);

    let conn = flame::connect(&ctx.endpoint).await?;

    let ssn_creation_start_time = Local::now();
    let ssn_attr = flame::SessionAttributes {
        application: app.clone(),
        slots,
    };
    let ssn = conn.create_session(&ssn_attr).await?;
    let ssn_creation_end_time = Local::now();

    let ssn_creation_time =
        ssn_creation_end_time.timestamp_millis() - ssn_creation_start_time.timestamp_millis();
    println!("Session <{}> was created in <{ssn_creation_time} ms>, start to run <{}> tasks in the session:\n", ssn.id, HumanCount(task_num as u64));

    let mut tasks = vec![];
    let tasks_creations_start_time = Local::now();

    let info = Arc::new(Mutex::new(BarInfo::new(task_num as u64)));

    for _ in 0..task_num {
        tasks.push(ssn.run_task(None, info.clone()));
    }

    try_join_all(tasks).await?;
    let tasks_creation_end_time = Local::now();

    {
        let mut informer = flame::lock_ptr!(info)?;
        informer.finish();
    }

    let tasks_creation_time =
        tasks_creation_end_time.timestamp_millis() - tasks_creations_start_time.timestamp_millis();

    println!(
        "\n\n<{}> tasks was completed in <{} ms>.\n",
        HumanCount(task_num as u64),
        HumanCount(tasks_creation_time as u64)
    );

    ssn.close().await?;

    Ok(())
}

struct BarInfo {
    bar: ProgressBar,
}

impl BarInfo {
    pub fn new(n: u64) -> Self {
        let bar = ProgressBar::new(n);
        bar.set_style(
            ProgressStyle::with_template("[{percent}%] {bar:45.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("=> "),
        );

        BarInfo { bar }
    }
    pub fn finish(&mut self) {
        self.bar.finish();
    }
}

impl flame::TaskInformer for BarInfo {
    fn on_update(&mut self, task: flame::Task) {
        if task.is_completed() {
            self.bar.inc(1)
        }
    }

    fn on_error(&mut self, _: FlameError) {
        print!("Got an error")
    }
}
