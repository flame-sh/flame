# /// script
# dependencies = [
#   "flame",
# ]
# [tool.uv.sources]
# flame = { path = ".." }
# ///
"""
Example usage of the Flame Python SDK service functionality.
"""

import asyncio
import flamepy

class ExampleService(flamepy.FlameService):
    """Custom implementation of GrpcShimService."""
    
    def __init__(self):
        self._session_context = None
        self._task_count = 0
    
    async def on_session_enter(self, context: flamepy.SessionContext) -> bool:
        """Handle session enter."""
        print(f"🟢 Entering session: {context.session_id}")
        print(f"   Application: {context.application.name}")
        print(f"   Shim: {context.application.shim}")
        print(f"   Common data: {context.common_data}")
        
        self._session_context = context
        self._task_count = 0
        
        return True
    
    async def on_task_invoke(self, context: flamepy.TaskContext) -> flamepy.TaskOutput:
        """Handle task invoke."""
        self._task_count += 1
        print(f"🟡 Invoking task {self._task_count}: {context.task_id}")
        print(f"   Session: {context.session_id}")
        
        if context.input:
            print(f"   Input: {context.input}")
        
        # Process the input and generate output
        if context.input:
            # Echo the input back
            output_data = context.input
        else:
            # Generate a simple response
            output_data = f"Task {self._task_count} completed successfully!".encode()
        
        print(f"   Output: {output_data}")
        
        return flamepy.TaskOutput(data=output_data)
    
    async def on_session_leave(self) -> bool:
        """Handle session leave."""
        print(f"🔴 Leaving session")
        print(f"   Total tasks processed: {self._task_count}")
        
        self._session_context = None
        
        return True


async def main():
    """Example main function."""
    print("🚀 Starting Flame Service Example")
    print("=" * 50)

    try:
        # Run the service
        await flamepy.run(ExampleService())
    except KeyboardInterrupt:
        print("\n🛑 Server stopped by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(main()) 