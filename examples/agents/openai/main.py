# /// script
# dependencies = [
#   "openai",
#   "flame",
# ]
# [tool.uv.sources]
# flame = { path = "/usr/local/flame/sdk/python" }
# ///

import os
import asyncio
import flame
from flame import FlameService, SessionContext, TaskContext, TaskOutput
from openai import OpenAI
import logging

logger = logging.getLogger(__name__)

class OpenAIAgent(FlameService):
    def __init__(self):
        self.session = None
        self.client = None
        self.system_prompt = None

    async def on_session_enter(self, context: SessionContext):
        self.client = OpenAI(api_key=os.getenv("DEEPSEEK_API_KEY"), base_url="https://api.deepseek.com")
        self.system_prompt = context.common_data.decode("utf-8")

    async def on_task_invoke(self, context: TaskContext) -> TaskOutput:
        logger.debug(f"Invoking task {context.task_id}")
        logger.debug(f"   Session: {context.session_id}")
        logger.debug(f"   Input: {context.input}")
        
        response = self.client.chat.completions.create(
            model="deepseek-chat",
            messages=[
                {"role": "system", "content": self.system_prompt},
                {"role": "user", "content": context.input.decode("utf-8")}
            ],
            )
        logger.debug(f"Response: {response.choices[0].message.content}")
        return TaskOutput(data=response.choices[0].message.content.encode("utf-8"))

    async def on_session_leave(self):
        pass

async def main():
    logging.basicConfig(level=logging.DEBUG)
    await flame.run(OpenAIAgent())

if __name__ == "__main__":
    asyncio.run(main())
