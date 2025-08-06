#! /usr/bin/env python3

import flame
import asyncio

LANGCHAIN_APP_NAME = "langchain-agent"


async def main():
    conn = await flame.connect("http://127.0.0.1:8080")
    session = await conn.create_session(LANGCHAIN_APP_NAME, 1)
    task = await session.run_task(b"task input data", flame.TaskInformer())
    print(task)

if __name__ == "__main__":
    asyncio.run(main())