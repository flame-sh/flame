#!/usr/bin/env python3
"""
Tests for the Flame Python SDK.
"""

import pytest
import asyncio
from unittest.mock import Mock, AsyncMock
from datetime import datetime

from flame import (
    Connection, SessionAttributes, ApplicationAttributes,
    Session, Task, Application, Shim, SessionState, TaskState,
    ApplicationState, FlameError, FlameErrorCode, TaskInformer
)


class TestTypes:
    """Test type definitions."""
    
    def test_session_attributes(self):
        """Test SessionAttributes."""
        attrs = SessionAttributes(
            application="test-app",
            slots=2,
            common_data=b"test data"
        )
        assert attrs.application == "test-app"
        assert attrs.slots == 2
        assert attrs.common_data == b"test data"
    
    def test_application_attributes(self):
        """Test ApplicationAttributes."""
        attrs = ApplicationAttributes(
            name="test-app",
            shim=Shim.SHELL,
            command="python",
            arguments=["script.py"],
            environments=["PATH=/usr/bin"],
            working_directory="/tmp"
        )
        assert attrs.name == "test-app"
        assert attrs.shim == Shim.SHELL
        assert attrs.command == "python"
        assert attrs.arguments == ["script.py"]
        assert attrs.environments == ["PATH=/usr/bin"]
        assert attrs.working_directory == "/tmp"
    
    def test_session(self):
        """Test Session."""
        session = Session(
            id="session-1",
            application="test-app",
            slots=2,
            state=SessionState.OPEN,
            creation_time=datetime.now(),
            pending=1,
            running=1,
            succeed=0,
            failed=0
        )
        assert session.id == "session-1"
        assert session.application == "test-app"
        assert session.slots == 2
        assert session.state == SessionState.OPEN
    
    def test_task(self):
        """Test Task."""
        task = Task(
            id="task-1",
            session_id="session-1",
            state=TaskState.PENDING,
            creation_time=datetime.now(),
            input=b"test input"
        )
        assert task.id == "task-1"
        assert task.session_id == "session-1"
        assert task.state == TaskState.PENDING
        assert task.input == b"test input"
        assert not task.is_completed()
        
        # Test completed task
        completed_task = Task(
            id="task-2",
            session_id="session-1",
            state=TaskState.SUCCEED,
            creation_time=datetime.now()
        )
        assert completed_task.is_completed()
    
    def test_application(self):
        """Test Application."""
        app = Application(
            id="app-1",
            name="test-app",
            shim=Shim.SHELL,
            state=ApplicationState.ENABLED,
            creation_time=datetime.now(),
            command="python",
            arguments=["script.py"]
        )
        assert app.id == "app-1"
        assert app.name == "test-app"
        assert app.shim == Shim.SHELL
        assert app.state == ApplicationState.ENABLED


class TestFlameError:
    """Test FlameError."""
    
    def test_flame_error(self):
        """Test FlameError creation and string representation."""
        error = FlameError(FlameErrorCode.INVALID_CONFIG, "Test error")
        assert error.code == FlameErrorCode.INVALID_CONFIG
        assert error.message == "Test error"
        assert "Test error" in str(error)
        assert "INVALID_CONFIG" in str(error)


class TestTaskInformer:
    """Test TaskInformer."""
    
    def test_task_informer_default(self):
        """Test default TaskInformer implementation."""
        informer = TaskInformer()
        task = Task(
            id="task-1",
            session_id="session-1",
            state=TaskState.PENDING,
            creation_time=datetime.now()
        )
        error = FlameError(FlameErrorCode.INTERNAL, "Test error")
        
        # Should not raise any exceptions
        informer.on_update(task)
        informer.on_error(error)


class TestConnection:
    """Test Connection class."""
    
    @pytest.mark.asyncio
    async def test_connection_connect_empty_addr(self):
        """Test connection with empty address."""
        with pytest.raises(FlameError) as exc_info:
            await Connection.connect("")
        assert exc_info.value.code == FlameErrorCode.INVALID_CONFIG
        assert "address cannot be empty" in exc_info.value.message
    
    @pytest.mark.asyncio
    async def test_connection_connect_invalid_addr(self):
        """Test connection with invalid address."""
        with pytest.raises(FlameError) as exc_info:
            await Connection.connect("invalid://address")
        assert exc_info.value.code == FlameErrorCode.INVALID_CONFIG


class TestEnums:
    """Test enum values."""
    
    def test_session_state_values(self):
        """Test SessionState enum values."""
        assert SessionState.OPEN == 0
        assert SessionState.CLOSED == 1
    
    def test_task_state_values(self):
        """Test TaskState enum values."""
        assert TaskState.PENDING == 0
        assert TaskState.RUNNING == 1
        assert TaskState.SUCCEED == 2
        assert TaskState.FAILED == 3
    
    def test_application_state_values(self):
        """Test ApplicationState enum values."""
        assert ApplicationState.ENABLED == 0
        assert ApplicationState.DISABLED == 1
    
    def test_shim_values(self):
        """Test Shim enum values."""
        assert Shim.LOG == 0
        assert Shim.STDIO == 1
        assert Shim.WASM == 2
        assert Shim.SHELL == 3
        assert Shim.GRPC == 4
    
    def test_flame_error_code_values(self):
        """Test FlameErrorCode enum values."""
        assert FlameErrorCode.INVALID_CONFIG == 0
        assert FlameErrorCode.INVALID_STATE == 1
        assert FlameErrorCode.INVALID_ARGUMENT == 2
        assert FlameErrorCode.INTERNAL == 3


if __name__ == "__main__":
    pytest.main([__file__]) 