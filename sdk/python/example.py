#!/usr/bin/env python3
"""
Example usage of the Flame Python SDK.
"""

import asyncio
from flame import Connection, SessionAttributes, Shim, TaskInformer


class MyTaskInformer(TaskInformer):
    """Example task informer that prints task updates."""
    
    def on_update(self, task):
        print(f"Task {task.id}: {task.state}")
    
    def on_error(self, error):
        print(f"Error: {error}")


async def main():
    """Example main function."""
    try:
        # Connect to Flame service
        print("Connecting to Flame service...")
        conn = await Connection.connect("http://127.0.0.1:8080")

        # Create a session
        print("Creating session...")
        session = await conn.create_session(SessionAttributes(
            application="flmexec",
            slots=1,
            common_data=b"shared data"
        ))
        
        print(f"Created session: {session.id}")
        # Create and run a task
        print("Creating task...")
        task = await session.create_task(b"task input data")
        
        print(f"Created task: {task.id}")
        
        # Watch task progress
        print("Watching task progress...")
        await session.run_task(b"task input data", MyTaskInformer())
        
        # Close session
        print("Closing session...")
        await conn.close_session(session.id)
        
        # Close connection
        await conn.close()
        
        print("Example completed successfully!")
        
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main()) 