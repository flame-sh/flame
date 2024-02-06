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

use rand::distributions::{Distribution, Uniform};
use std::error::Error;

use clap::Parser;

#[derive(Parser)]
#[command(name = "pi-local")]
#[command(author = "Klaus Ma <klaus@xflops.cn>")]
#[command(version = "0.1.0")]
#[command(about = "Flame Pi Local Example", long_about = None)]
struct Cli {
    #[arg(long)]
    point_num: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut rng = rand::thread_rng();
    let die = Uniform::from(0.0..1.0);

    let total = cli.point_num;
    let mut area = 0.0;

    for _ in 0..total as i64 {
        let x: f64 = die.sample(&mut rng);
        let y: f64 = die.sample(&mut rng);
        let dist = (x * x + y * y).sqrt();

        if dist <= 1.0 {
            area += 1.0;
        }
    }

    let pi = 4_f64 * area / total;
    println!("pi = 4*({}/{}) = {}", area, total, pi);

    Ok(())
}
