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
use std::sync::{Arc, Mutex};

use clap::Parser;
use futures::future::try_join_all;
use rand::Rng;
use serde::{Deserialize, Serialize};

use self::flame::{FlameError, SessionAttributes, Task, TaskInformer, TaskInput};
use flame_client as flame;

#[derive(Parser)]
#[command(name = "matrix")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Matrix Example", long_about = None)]
struct Cli {
    #[arg(short = 'n', long)]
    size: u16,
    #[arg(short, long)]
    app: Option<String>,
    #[arg(short, long)]
    slots: Option<i32>,
}

const DEFAULT_APP: &str = "matrix";
const DEFAULT_SLOTS: i32 = 1;

#[derive(Default, Serialize, Deserialize)]
struct MatrixData {
    n: u16,
    u: Vec<Vec<i32>>,
    v: Vec<Vec<i32>>,
}

impl MatrixData {
    pub fn random(size: u16) -> Self {
        let mut data = MatrixData {
            n: size,
            ..MatrixData::default()
        };

        let mut rng = rand::thread_rng();

        for _ in 0..data.n {
            let mut u = vec![];
            for _ in 0..data.n {
                u.push(rng.gen_range(0..10));
            }
            data.u.push(u);
        }

        for _ in 0..data.n {
            let mut v = vec![];
            for _ in 0..data.n {
                v.push(rng.gen_range(0..10));
            }
            data.v.push(v);
        }

        data
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let cli = Cli::parse();

    let host = match std::env::var("FLAME_SESSION_MANAGER_SERVICE_HOST") {
        Ok(ip) => ip,
        Err(_) => "127.0.0.1".to_string(),
    };

    let port = match std::env::var("FLAME_SESSION_MANAGER_SERVICE_PORT") {
        Ok(p) => p,
        Err(_) => "8080".to_string(),
    };

    let conn = flame::connect(format!("http://{}:{}", host, port).as_str()).await?;

    let app = cli.app.unwrap_or(DEFAULT_APP.to_string());
    let slots = cli.slots.unwrap_or(DEFAULT_SLOTS);

    let data = MatrixData::random(cli.size);
    let common_data =
        serde_json::to_string(&data).map_err(|e| FlameError::Internal(e.to_string()))?;

    for i in 0..cli.size {
        for j in 0..cli.size {
            print!("{} ", data.u[i as usize][j as usize]);
        }
        println!();
    }

    println!();
    println!("*");
    println!();

    for i in 0..cli.size {
        for j in 0..cli.size {
            print!("{} ", data.v[i as usize][j as usize]);
        }
        println!();
    }

    let ssn = conn
        .create_session(&SessionAttributes {
            application: app,
            slots,
            common_data: Some(common_data.into()),
        })
        .await?;

    let informer = Arc::new(Mutex::new(MatrixInfo::new(cli.size)));
    let mut tasks = vec![];
    for i in 0..cli.size * cli.size {
        tasks.push(ssn.run_task(
            Some(TaskInput::from(i.to_ne_bytes().to_vec())),
            informer.clone(),
        ));
    }

    println!();
    println!("=");
    println!();

    try_join_all(tasks).await?;

    {
        let informer = flame::lock_ptr!(informer)?;
        for i in 0..cli.size {
            for j in 0..cli.size {
                print!("{} ", informer.data[i as usize][j as usize]);
            }
            println!();
        }
    }

    ssn.close().await?;

    Ok(())
}

pub struct MatrixInfo {
    pub n: u16,
    pub data: Vec<Vec<i32>>,
}

impl MatrixInfo {
    pub fn new(size: u16) -> Self {
        let mut data = Vec::with_capacity(size.into());
        for _ in 0..size {
            data.push(vec![-1; size.into()]);
        }

        Self { n: size, data }
    }
}

impl TaskInformer for MatrixInfo {
    fn on_update(&mut self, task: Task) {
        if let Some(output) = task.output {
            let o = i32::from_ne_bytes(output.to_vec().try_into().unwrap());
            let task_id = task.id.parse::<u16>().unwrap() - 1;

            let (m, n) = ((task_id / self.n) as usize, (task_id % self.n) as usize);

            self.data[m][n] = o;
        }
    }

    fn on_error(&mut self, e: FlameError) {
        print!("Got an error: {}", e);
    }
}
