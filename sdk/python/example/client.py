#!/usr/bin/env python3
"""
Example usage of the Flame Python SDK.
"""

import asyncio
import flame

class MyTaskInformer(flame.TaskInformer):
    """Example task informer that prints task updates."""
    
    def on_update(self, task):
        print(f"Task {task.id}: {task.state.name}")
    
    def on_error(self, error):
        print(f"Error: {error}")


async def main():
    """Example main function."""
    try:
        # Create a session
        print("Creating session...")
        session = await flame.create_session(
            application="flmtest",
            common_data=b"shared data"
        )
        print(f"Created session: {session.id}")
        
        # Invoke task
        print("Running task...")
        await session.invoke(b"task input data", MyTaskInformer())
        
        # Close session
        print("Closing session...")
        await session.close()
        
        print("Example completed successfully!")
        
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main()) 