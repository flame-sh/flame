#! /usr/bin/env python3

import flame
import asyncio

OPENAI_APP_NAME = "openai-agent"

async def main():
    session = await flame.create_session(OPENAI_APP_NAME, b"You are a weather forecaster.")
    task = await session.invoke(b"Who are you?")
    print(task.output.decode("utf-8"))

if __name__ == "__main__":
    asyncio.run(main())