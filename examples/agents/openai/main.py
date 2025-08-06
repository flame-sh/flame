#! /usr/bin/env python3

import flame
from flame.service import FlameService, SessionContext, TaskContext, TaskOutput
from openai import OpenAI

class OpenAIAgent(FlameService):
    def __init__(self):
        self.session = None

    async def on_session_enter(self, context: SessionContext) -> bool:
        self.session = context.session
        return True

    async def on_task_invoke(self, context: TaskContext) -> TaskOutput:
        return TaskOutput(data=b"task output data")

    async def on_session_leave(self) -> bool:
        return True

if __name__ == "__main__":
    flame.run(OpenAIAgent())
