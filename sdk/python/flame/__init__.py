"""
Copyright 2025 The Flame Authors.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
"""

from .types import (
    # Type aliases
    TaskID,
    SessionID,
    ApplicationID,
    Message,
    TaskInput,
    TaskOutput,
    CommonData,
    
    # Enums
    SessionState,
    TaskState,
    ApplicationState,
    Shim,
    FlameErrorCode,
    
    # Classes
    FlameError,
    SessionAttributes,
    ApplicationAttributes,
    Session,
    Task,
    Application,
    FlameContext,
    TaskInformer,
)

# Import client classes only if grpc is available
try:
    from .client import Connection, Session, TaskWatcher, connect
    from .service import (
        FlameService,
        ApplicationContext, SessionContext, TaskContext, TaskOutput,
        run
    )
    _CLIENT_AVAILABLE = True
except ImportError:
    _CLIENT_AVAILABLE = False
    # Create placeholder classes for when grpc is not available
    class Connection:
        """Placeholder for Connection when grpc is not available."""
        pass

    class Session:
        """Placeholder for Session when grpc is not available."""
        pass
    
    class TaskWatcher:
        """Placeholder for TaskWatcher when grpc is not available."""
        pass
    
    # Placeholder service classes
    class FlameService:
        """Placeholder for FlameService when grpc is not available."""
        pass
    
    class ApplicationContext:
        """Placeholder for ApplicationContext when grpc is not available."""
        def __init__(self, name=None, shim=None, url=None, command=None, **kwargs):
            self.name = name
            self.shim = shim
            self.url = url
            self.command = command
    
    class SessionContext:
        """Placeholder for SessionContext when grpc is not available."""
        def __init__(self, session_id=None, application=None, common_data=None, **kwargs):
            self.session_id = session_id
            self.application = application
            self.common_data = common_data
    
    class TaskContext:
        """Placeholder for TaskContext when grpc is not available."""
        def __init__(self, task_id=None, session_id=None, input=None, **kwargs):
            self.task_id = task_id
            self.session_id = session_id
            self.input = input
    
    class TaskOutput:
        """Placeholder for TaskOutput when grpc is not available."""
        def __init__(self, data=None, **kwargs):
            self.data = data
    
__version__ = "0.1.0"

__all__ = [
    # Type aliases
    "TaskID",
    "SessionID", 
    "ApplicationID",
    "Message",
    "TaskInput",
    "TaskOutput",
    "CommonData",
    
    # Enums
    "SessionState",
    "TaskState", 
    "ApplicationState",
    "Shim",
    "FlameErrorCode",
    
    # Classes
    "FlameError",
    "SessionAttributes",
    "ApplicationAttributes", 
    "Session",
    "Task",
    "Application",
    "FlameContext",
    "TaskInformer",
    
    # Client classes
    "Connection",
    "connect",
    "TaskWatcher",
    "Session", 
    "Task",
    "TaskInput",
    "TaskOutput",
    "CommonData",
    
    # Service classes
    "FlameService",
    "ApplicationContext",
    "SessionContext",
    "TaskContext",
    "TaskOutput",
    "run",
] 