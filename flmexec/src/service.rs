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

mod api;
mod script;


use flame_rs::{
    self as flame,
    apis::{FlameError, TaskOutput},
    service::{SessionContext, TaskContext},
    trace::TraceFn,
    trace_fn,
};

use api::Script;

#[derive(Clone)]
pub struct FlmexecService {}

#[tonic::async_trait]
impl flame::service::FlameService for FlmexecService {
    async fn on_session_enter(&self, _: SessionContext) -> Result<(), FlameError> {
        trace_fn!("FlmexecService::on_session_enter");
        Ok(())
    }

    async fn on_task_invoke(&self, ctx: TaskContext) -> Result<Option<TaskOutput>, FlameError> {
        trace_fn!("FlmexecService::on_task_invoke");

        log::debug!("Try to get task input from context");
        let input = ctx
            .input
            .as_ref()
            .ok_or(FlameError::Internal("No task input".to_string()))?;
        log::debug!("Try to parse script from input:\n{}", String::from_utf8_lossy(input));
        let script: Script =
            serde_json::from_slice(input).map_err(|e| FlameError::Internal(format!("failed to parse script: {e}")))?;
        log::debug!("Try to create engine for script: {:?}", script);
        let engine = script::new(&script)?;
        log::debug!("Created engine for language: {}", script.language);
        log::debug!("Code:\n{}", script.code);
        // log::debug!("Input:\n{}", script.input.as_ref().map(|x| String::from_utf8_lossy(x)).unwrap_or("None".to_string()));
        let output = engine.run()?;
        // log::debug!("Output:\n{}", output.as_ref().map(|x| String::from_utf8_lossy(x)).unwrap_or("None".to_string()));

        Ok(output.map(TaskOutput::from))
    }

    async fn on_session_leave(&self) -> Result<(), FlameError> {
        trace_fn!("FlmexecService::on_session_leave");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    flame::service::run(FlmexecService {}).await?;

    log::debug!("FlmexecService was stopped.");

    Ok(())
}
