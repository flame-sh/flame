#!/usr/bin/env python3
"""
Basic test for the Flame Python SDK service functionality.
"""

import asyncio

def test_service_types():
    """Test service type definitions."""
    try:
        from flame import (
            GrpcShimService, SimpleGrpcShimService, GrpcShimServer,
            ApplicationContext, SessionContext, TaskContext, TaskOutput,
            run_shim_server, create_simple_shim_service, Shim
        )
        
        # Test ApplicationContext
        app_ctx = ApplicationContext(
            name="test-app",
            shim=Shim.GRPC,
            url="http://localhost:8080",
            command="python"
        )
        assert app_ctx.name == "test-app"
        assert app_ctx.shim == Shim.GRPC
        print("✅ ApplicationContext test passed")
        
        # Test SessionContext
        session_ctx = SessionContext(
            session_id="session-1",
            application=app_ctx,
            common_data=b"shared data"
        )
        assert session_ctx.session_id == "session-1"
        assert session_ctx.application == app_ctx
        assert session_ctx.common_data == b"shared data"
        print("✅ SessionContext test passed")
        
        # Test TaskContext
        task_ctx = TaskContext(
            task_id="task-1",
            session_id="session-1",
            input=b"task input"
        )
        assert task_ctx.task_id == "task-1"
        assert task_ctx.session_id == "session-1"
        assert task_ctx.input == b"task input"
        print("✅ TaskContext test passed")
        
        # Test TaskOutput
        output = TaskOutput(data=b"task output")
        assert output.data == b"task output"
        print("✅ TaskOutput test passed")
        
        return True
        
    except Exception as e:
        print(f"❌ Service types test failed: {e}")
        return False


async def test_simple_service():
    """Test SimpleGrpcShimService."""
    try:
        from flame import SimpleGrpcShimService, ApplicationContext, SessionContext, TaskContext, Shim
        
        service = SimpleGrpcShimService()
        
        # Test session enter
        app_ctx = ApplicationContext(name="test-app", shim=Shim.GRPC)
        session_ctx = SessionContext(
            session_id="session-1",
            application=app_ctx,
            common_data=b"shared data"
        )
        
        result = await service.on_session_enter(session_ctx)
        assert result is True
        assert service._session_active is True
        print("✅ Session enter test passed")
        
        # Test task invoke
        task_ctx = TaskContext(
            task_id="task-1",
            session_id="session-1",
            input=b"hello world"
        )
        
        output = await service.on_task_invoke(task_ctx)
        assert output.data == b"hello world"
        print("✅ Task invoke test passed")
        
        # Test session leave
        result = await service.on_session_leave()
        assert result is True
        assert service._session_active is False
        print("✅ Session leave test passed")
        
        return True
        
    except Exception as e:
        print(f"❌ Simple service test failed: {e}")
        return False


async def test_custom_service():
    """Test custom GrpcShimService implementation."""
    try:
        from flame import GrpcShimService, ApplicationContext, SessionContext, TaskContext, Shim
        
        class TestGrpcShimService(GrpcShimService):
            """Test implementation of GrpcShimService."""
            
            def __init__(self):
                self.session_entered = False
                self.tasks_processed = 0
                self.session_left = False
            
            async def on_session_enter(self, context):
                self.session_entered = True
                return True
            
            async def on_task_invoke(self, context):
                self.tasks_processed += 1
                from flame import TaskOutput
                return TaskOutput(data=f"Task {self.tasks_processed}".encode())
            
            async def on_session_leave(self):
                self.session_left = True
                return True
        
        service = TestGrpcShimService()
        
        # Test session enter
        app_ctx = ApplicationContext(name="test-app", shim=Shim.GRPC)
        session_ctx = SessionContext(session_id="session-1", application=app_ctx)
        
        result = await service.on_session_enter(session_ctx)
        assert result is True
        assert service.session_entered is True
        print("✅ Custom service session enter test passed")
        
        # Test task invoke
        task_ctx = TaskContext(task_id="task-1", session_id="session-1")
        output = await service.on_task_invoke(task_ctx)
        assert output.data == b"Task 1"
        assert service.tasks_processed == 1
        print("✅ Custom service task invoke test passed")
        
        # Test session leave
        result = await service.on_session_leave()
        assert result is True
        assert service.session_left is True
        print("✅ Custom service session leave test passed")
        
        return True
        
    except Exception as e:
        print(f"❌ Custom service test failed: {e}")
        return False


def test_service_functions():
    """Test service utility functions."""
    try:
        from flame import create_simple_shim_service, SimpleGrpcShimService
        
        service = create_simple_shim_service()
        assert isinstance(service, SimpleGrpcShimService)
        print("✅ Service functions test passed")
        
        return True
        
    except Exception as e:
        print(f"❌ Service functions test failed: {e}")
        return False


async def main():
    """Run all service tests."""
    print("Testing Flame Python SDK Service Functionality...")
    print("=" * 50)
    
    success = True
    success &= test_service_types()
    success &= await test_simple_service()
    success &= await test_custom_service()
    success &= test_service_functions()
    
    if success:
        print("\n🎉 All service tests passed! The service functionality is working correctly.")
    else:
        print("\n❌ Some service tests failed. Please check the errors above.")


if __name__ == "__main__":
    asyncio.run(main()) 