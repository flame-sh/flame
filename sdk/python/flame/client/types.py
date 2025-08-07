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

from dataclasses import dataclass
from enum import IntEnum
from typing import Optional, List, Dict, Any, Union
from datetime import datetime


# Type aliases
TaskID = str
SessionID = str
ApplicationID = str
Message = bytes
TaskInput = Message
TaskOutput = Message
CommonData = Message


# Constants
DEFAULT_FLAME_CONF = "flame-conf.yaml"
DEFAULT_CONTEXT_NAME = "flame"
DEFAULT_FLAME_ENDPOINT = "http://127.0.0.1:8080"


class SessionState(IntEnum):
    """Session state enumeration."""
    OPEN = 0
    CLOSED = 1


class TaskState(IntEnum):
    """Task state enumeration."""
    PENDING = 0
    RUNNING = 1
    SUCCEED = 2
    FAILED = 3


class ApplicationState(IntEnum):
    """Application state enumeration."""
    ENABLED = 0
    DISABLED = 1


class Shim(IntEnum):
    """Shim type enumeration."""
    LOG = 0
    STDIO = 1
    WASM = 2
    SHELL = 3
    GRPC = 4


class FlameErrorCode(IntEnum):
    """Flame error code enumeration."""
    INVALID_CONFIG = 0
    INVALID_STATE = 1
    INVALID_ARGUMENT = 2
    INTERNAL = 3


class FlameError(Exception):
    """Flame SDK error exception."""
    
    def __init__(self, code: FlameErrorCode, message: str):
        self.code = code
        self.message = message
        super().__init__(f"{message} (code: {code})")


@dataclass
class SessionAttributes:
    """Attributes for creating a session."""
    application: str
    slots: int
    common_data: Optional[bytes] = None


@dataclass
class ApplicationAttributes:
    """Attributes for an application."""
    name: str
    shim: Shim
    url: Optional[str] = None
    command: Optional[str] = None
    arguments: Optional[List[str]] = None
    environments: Optional[List[str]] = None
    working_directory: Optional[str] = None




@dataclass
class Task:
    """Represents a computing task."""
    id: TaskID
    session_id: SessionID
    state: TaskState
    creation_time: datetime
    input: Optional[bytes] = None
    output: Optional[bytes] = None
    completion_time: Optional[datetime] = None
    
    def is_completed(self) -> bool:
        """Check if the task is completed."""
        return self.state in (TaskState.SUCCEED, TaskState.FAILED)


@dataclass
class Application:
    """Represents a distributed application."""
    id: ApplicationID
    name: str
    shim: Shim
    state: ApplicationState
    creation_time: datetime
    url: Optional[str] = None
    command: Optional[str] = None
    arguments: Optional[List[str]] = None
    environments: Optional[List[str]] = None
    working_directory: Optional[str] = None


@dataclass
class FlameContext:
    """Flame context configuration."""
    name: str
    endpoint: str
    
    @classmethod
    def default(cls) -> "FlameContext":
        """Create a default Flame context."""
        return cls(
            name=DEFAULT_CONTEXT_NAME,
            endpoint=DEFAULT_FLAME_ENDPOINT
        )


class TaskInformer:
    """Interface for task updates."""
    
    def on_update(self, task: Task) -> None:
        """Called when a task is updated."""
        pass
    
    def on_error(self, error: FlameError) -> None:
        """Called when an error occurs."""
        pass 