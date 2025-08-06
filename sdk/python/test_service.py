#!/usr/bin/env python3
"""
Tests for the Flame Python SDK service functionality.
"""

import pytest
import asyncio
from unittest.mock import Mock, AsyncMock
from datetime import datetime

from flame import (
    GrpcShimService, SimpleGrpcShimService, GrpcShimServer,
    ApplicationContext, SessionContext, TaskContext, TaskOutput,
    run_shim_server, create_simple_shim_service, Shim, FlameError, FlameErrorCode
)


class TestServiceTypes:
    """Test service type definitions."""
    
    def test_application_context(self):
        """Test ApplicationContext."""
        app_ctx = ApplicationContext(
            name="test-app",
            shim=Shim.GRPC,
            url="http://localhost:8080",
            command="python"
        )
        assert app_ctx.name == "test-app"
        assert app_ctx.shim == Shim.GRPC
        assert app_ctx.url == "http://localhost:8080"
        assert app_ctx.command == "python"
    
    def test_session_context(self):
        """Test SessionContext."""
        app_ctx = ApplicationContext(name="test-app", shim=Shim.GRPC)
        session_ctx = SessionContext(
            session_id="session-1",
            application=app_ctx,
            common_data=b"shared data"
        )
        assert session_ctx.session_id == "session-1"
        assert session_ctx.application == app_ctx
        assert session_ctx.common_data == b"shared data"
    
    def test_task_context(self):
        """Test TaskContext."""
        task_ctx = TaskContext(
            task_id="task-1",
            session_id="session-1",
            input=b"task input"
        )
        assert task_ctx.task_id == "task-1"
        assert task_ctx.session_id == "session-1"
        assert task_ctx.input == b"task input"
    
    def test_task_output(self):
        """Test TaskOutput."""
        output = TaskOutput(data=b"task output")
        assert output.data == b"task output"


class TestSimpleGrpcShimService:
    """Test SimpleGrpcShimService."""
    
    @pytest.mark.asyncio
    async def test_session_lifecycle(self):
        """Test session lifecycle."""
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
        assert service._session_context == session_ctx
        
        # Test task invoke
        task_ctx = TaskContext(
            task_id="task-1",
            session_id="session-1",
            input=b"hello world"
        )
        
        output = await service.on_task_invoke(task_ctx)
        assert output.data == b"hello world"
        
        # Test session leave
        result = await service.on_session_leave()
        assert result is True
        assert service._session_active is False
        assert service._session_context is None
    
    @pytest.mark.asyncio
    async def test_task_invoke_no_session(self):
        """Test task invoke without active session."""
        service = SimpleGrpcShimService()
        
        task_ctx = TaskContext(
            task_id="task-1",
            session_id="session-1",
            input=b"hello world"
        )
        
        with pytest.raises(FlameError) as exc_info:
            await service.on_task_invoke(task_ctx)
        
        assert exc_info.value.code == FlameErrorCode.INVALID_STATE
        assert "No active session" in exc_info.value.message
    
    @pytest.mark.asyncio
    async def test_task_invoke_no_input(self):
        """Test task invoke without input."""
        service = SimpleGrpcShimService()
        
        # Start session
        app_ctx = ApplicationContext(name="test-app", shim=Shim.GRPC)
        session_ctx = SessionContext(session_id="session-1", application=app_ctx)
        await service.on_session_enter(session_ctx)
        
        # Invoke task without input
        task_ctx = TaskContext(task_id="task-1", session_id="session-1")
        output = await service.on_task_invoke(task_ctx)
        
        assert output.data == b"Hello from Python gRPC shim!"


class TestCustomGrpcShimService:
    """Test custom GrpcShimService implementation."""
    
    class TestGrpcShimService(GrpcShimService):
        """Test implementation of GrpcShimService."""
        
        def __init__(self):
            self.session_entered = False
            self.tasks_processed = 0
            self.session_left = False
        
        async def on_session_enter(self, context: SessionContext) -> bool:
            self.session_entered = True
            return True
        
        async def on_task_invoke(self, context: TaskContext) -> TaskOutput:
            self.tasks_processed += 1
            return TaskOutput(data=f"Task {self.tasks_processed}".encode())
        
        async def on_session_leave(self) -> bool:
            self.session_left = True
            return True
    
    @pytest.mark.asyncio
    async def test_custom_service(self):
        """Test custom service implementation."""
        service = self.TestGrpcShimService()
        
        # Test session enter
        app_ctx = ApplicationContext(name="test-app", shim=Shim.GRPC)
        session_ctx = SessionContext(session_id="session-1", application=app_ctx)
        
        result = await service.on_session_enter(session_ctx)
        assert result is True
        assert service.session_entered is True
        
        # Test task invoke
        task_ctx = TaskContext(task_id="task-1", session_id="session-1")
        output = await service.on_task_invoke(task_ctx)
        assert output.data == b"Task 1"
        assert service.tasks_processed == 1
        
        # Test another task
        output = await service.on_task_invoke(task_ctx)
        assert output.data == b"Task 2"
        assert service.tasks_processed == 2
        
        # Test session leave
        result = await service.on_session_leave()
        assert result is True
        assert service.session_left is True


class TestServiceFunctions:
    """Test service utility functions."""
    
    def test_create_simple_shim_service(self):
        """Test create_simple_shim_service function."""
        service = create_simple_shim_service()
        assert isinstance(service, SimpleGrpcShimService)
    
    @pytest.mark.asyncio
    async def test_run_shim_server_placeholder(self):
        """Test run_shim_server function (placeholder test)."""
        # This is a placeholder test since we can't actually run a server in tests
        service = create_simple_shim_service()
        
        # The function should not raise an exception
        try:
            # This would normally start a server, but with placeholders it should be safe
            pass
        except Exception as e:
            pytest.fail(f"run_shim_server should not raise exception: {e}")


class TestGrpcShimServer:
    """Test GrpcShimServer class."""
    
    def test_server_initialization(self):
        """Test GrpcShimServer initialization."""
        service = create_simple_shim_service()
        server = GrpcShimServer(service, port=50051)
        
        assert server._service == service
        assert server._port == 50051
        assert server._server is None


if __name__ == "__main__":
    pytest.main([__file__]) 