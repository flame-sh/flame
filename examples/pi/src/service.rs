/*
Copyright 2025 The Flame Authors.
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

mod util;

use rand::distr::{Distribution, Uniform};

use flame_rs::{
    self as flame,
    apis::{FlameError, TaskOutput},
    service::{SessionContext, TaskContext},
};

#[derive(Clone)]
pub struct PiService {}

#[tonic::async_trait]
impl flame::service::FlameService for PiService {
    async fn on_session_enter(&self, _: SessionContext) -> Result<(), FlameError> {
        Ok(())
    }

    async fn on_task_invoke(&self, ctx: TaskContext) -> Result<Option<TaskOutput>, FlameError> {
        let mut rng = rand::rng();
        let die = Uniform::try_from(0.0..1.0).unwrap();

        let input = ctx.input.unwrap_or(util::zero_u32());
        let total = util::bytes_to_u32(input.to_vec())?;
        let mut sum = 0u32;

        for _ in 0..total {
            let x: f64 = die.sample(&mut rng);
            let y: f64 = die.sample(&mut rng);
            let dist = (x * x + y * y).sqrt();

            if dist <= 1.0 {
                sum += 1;
            }
        }

        Ok(Some(TaskOutput::from(util::u32_to_bytes(sum))))
    }

    async fn on_session_leave(&self) -> Result<(), FlameError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    flame::service::run(PiService {}).await?;

    log::debug!("FlmpingService was stopped.");

    Ok(())
}
