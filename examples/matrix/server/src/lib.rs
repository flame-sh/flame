cargo_component_bindings::generate!();

use crate::bindings::exports::component::flame::service::{
    FlameError, Guest, SessionContext, TaskContext, TaskInput, TaskOutput,
};

struct Component {}

impl Guest for Component {
    fn on_session_enter(ctx: SessionContext) -> Result<(), FlameError> {
        println!("session <{}> enter", ctx.session_id);
        Ok(())
    }

    fn on_session_leave(ctx: SessionContext) -> Result<(), FlameError> {
        println!("session <{}> leave", ctx.session_id);
        Ok(())
    }

    fn on_task_invoke(
        ctx: TaskContext,
        _: Option<TaskInput>,
    ) -> Result<Option<TaskOutput>, FlameError> {
        println!("task <{}/{}> invoke", ctx.session_id, ctx.task_id);
        Ok(None)
    }
}
