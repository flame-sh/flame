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

use gethostname::gethostname;

use flame_service::{self as flame, FlameError, SessionContext, TaskContext, TaskOutput};

#[derive(Clone)]
pub struct FlmpingService {}

#[tonic::async_trait]
impl flame::FlameService for FlmpingService {
    async fn on_session_enter(&self, _: SessionContext) -> Result<(), FlameError> {
        Ok(())
    }

    async fn on_task_invoke(&self, ctx: TaskContext) -> Result<Option<TaskOutput>, FlameError> {
        Ok(Some(TaskOutput::from(format!(
            "Task <{}/{}> is executed on <{:?}>",
            ctx.session_id,
            ctx.task_id,
            gethostname(),
        ))))
    }

    async fn on_session_leave(&self) -> Result<(), FlameError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    flame_service::run(FlmpingService {}).await?;

    Ok(())
}
