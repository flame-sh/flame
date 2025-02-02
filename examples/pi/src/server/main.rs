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

use rand::distr::{Distribution, Uniform};

use std::error::Error;
use std::io::stdin;

fn main() -> Result<(), Box<dyn Error>> {
    let mut rng = rand::rng();
    let die = Uniform::try_from(0.0..1.0).unwrap();

    let mut input = String::new();
    stdin().read_line(&mut input)?;

    let total = input.trim().parse::<i32>()?;
    let mut sum = 0;

    for _ in 0..total {
        let x: f64 = die.sample(&mut rng);
        let y: f64 = die.sample(&mut rng);
        let dist = (x * x + y * y).sqrt();

        if dist <= 1.0 {
            sum += 1;
        }
    }

    println!("{}", sum);

    Ok(())
}
