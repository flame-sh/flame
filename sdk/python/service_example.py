#!/usr/bin/env python3
"""
Example usage of the Flame Python SDK service functionality.
"""

import asyncio
from flame import (
    GrpcShimService, SimpleGrpcShimService, GrpcShimServer,
    ApplicationContext, SessionContext, TaskContext, TaskOutput,
    run_shim_server, create_simple_shim_service, Shim
)


class CustomGrpcShimService(GrpcShimService):
    """Custom implementation of GrpcShimService."""
    
    def __init__(self):
        self._session_active = False
        self._session_context = None
        self._task_count = 0
    
    async def on_session_enter(self, context: SessionContext) -> bool:
        """Handle session enter."""
        print(f"🟢 Entering session: {context.session_id}")
        print(f"   Application: {context.application.name}")
        print(f"   Shim: {context.application.shim}")
        print(f"   Common data: {context.common_data}")
        
        self._session_context = context
        self._session_active = True
        self._task_count = 0
        
        return True
    
    async def on_task_invoke(self, context: TaskContext) -> TaskOutput:
        """Handle task invoke."""
        if not self._session_active:
            raise Exception("No active session")
        
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
        
        return TaskOutput(data=output_data)
    
    async def on_session_leave(self) -> bool:
        """Handle session leave."""
        print(f"🔴 Leaving session")
        print(f"   Total tasks processed: {self._task_count}")
        
        self._session_active = False
        self._session_context = None
        
        return True


async def main():
    """Example main function."""
    print("🚀 Starting Flame gRPC Shim Service Example")
    print("=" * 50)
    
    # Option 1: Use the simple service
    print("\n1. Using SimpleGrpcShimService:")
    simple_service = create_simple_shim_service()
    
    # Option 2: Use a custom service
    print("\n2. Using CustomGrpcShimService:")
    custom_service = CustomGrpcShimService()
    
    # Choose which service to run
    service = custom_service  # Change to simple_service to use the simple one
    
    print(f"\nStarting gRPC shim server on port 50051...")
    print("Press Ctrl+C to stop the server")
    
    try:
        # Run the server
        await run_shim_server(service, port=50051)
    except KeyboardInterrupt:
        print("\n🛑 Server stopped by user")
    except Exception as e:
        print(f"\n❌ Error: {e}")


if __name__ == "__main__":
    asyncio.run(main()) 