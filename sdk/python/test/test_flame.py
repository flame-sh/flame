# /// script
# dependencies = [
#   "asyncio",
#   "flamepy",
# ]
# [tool.uv.sources]
# flamepy = { path = ".." }
# ///


import asyncio
import flamepy 

class MyTaskInformer(flamepy.TaskInformer):
    """Example task informer that prints task updates."""
    
    def on_update(self, task):
        pass
    
    def on_error(self, error):
        pass

async def test_create_session():
    session = await flamepy.create_session(
        application="flmtest",
        common_data=b"shared data"
    )

    await session.invoke(b"task input data", MyTaskInformer())
    await session.close()
        
    print("Test create session completed successfully!")


async def test_invoke_multiple_tasks():
    session = await flamepy.create_session(
        application="flmtest",
        common_data=b"shared data"
    )

    for i in range(10):
        await session.invoke(b"task input data", MyTaskInformer())
    await session.close()

    print("Test invoke multiple tasks completed successfully!")


async def test_invoke_multiple_sessions():
    for i in range(10):
        session = await flamepy.create_session(
            application="flmtest",
            common_data=b"shared data"
        )

        for i in range(10):
            await session.invoke(b"task input data", MyTaskInformer())
        await session.close()

    print("Test invoke multiple sessions completed successfully!")




if __name__ == "__main__":
    asyncio.run(test_create_session()) 
    asyncio.run(test_invoke_multiple_tasks()) 
    asyncio.run(test_invoke_multiple_sessions()) 