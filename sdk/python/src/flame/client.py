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

import asyncio
import threading
from typing import Optional, List, Dict, Any, Union
from urllib.parse import urlparse
import grpc
import grpc.aio
import os

from datetime import datetime
from .types import (
    Task, Application, SessionAttributes, ApplicationAttributes,
    SessionID, TaskID, ApplicationID, TaskInput, TaskOutput, CommonData,
    SessionState, TaskState, ApplicationState, Shim, FlameError, FlameErrorCode,
    TaskInformer
)

from .types_pb2 import ApplicationSpec, SessionSpec, TaskSpec
from .frontend_pb2 import RegisterApplicationRequest, UnregisterApplicationRequest, ListApplicationRequest, CreateSessionRequest, ListSessionRequest, GetSessionRequest, CloseSessionRequest, CreateTaskRequest, WatchTaskRequest, GetTaskRequest
from .frontend_pb2_grpc import FrontendStub

async def connect(addr: str) -> "Connection":
    """Connect to the Flame service."""
    return await Connection.connect(addr)

async def create_session(application: str, common_data: Dict[str, Any], slots: int = 1) -> "Session":
    conn = await ConnectionInstance().instance()
    session = await conn.create_session(SessionAttributes(application=application, common_data=common_data, slots=slots))
    return session

class ConnectionInstance:
    """Connection instance."""
    _connection = None
    _lock = threading.Lock()

    async def instance(self) -> "Connection":
        """Get the connection instance."""
        with self._lock:
            if not self._connection:
                self._connection = await connect(os.getenv("FLAME_ENDPOINT", "http://127.0.0.1:8080"))
            return self._connection

class Connection:
    """Connection to the Flame service."""
    
    def __init__(self, addr: str, channel: grpc.aio.Channel, frontend: FrontendStub):
        self.addr = addr
        self._channel = channel
        self._frontend = frontend
    
    @classmethod
    async def connect(cls, addr: str) -> "Connection":
        """Establish a connection to the Flame service."""
        if not addr:
            raise FlameError(
                FlameErrorCode.INVALID_CONFIG,
                "address cannot be empty"
            )
        
        try:
            parsed_addr = urlparse(addr)
            host = parsed_addr.hostname or parsed_addr.path
            port = parsed_addr.port or 8080
            
            # Create insecure channel
            channel = grpc.aio.insecure_channel(f"{host}:{port}")
            
            # Wait for channel to be ready
            await channel.channel_ready()
            
            # Create frontend stub
            frontend = FrontendStub(channel)
            
            return cls(addr, channel, frontend)
            
        except Exception as e:
            raise FlameError(
                FlameErrorCode.INVALID_CONFIG,
                f"failed to connect to {addr}: {str(e)}"
            )
    
    async def close(self) -> None:
        """Close the connection."""
        await self._channel.close()
    
    async def register_application(
        self, 
        name: str, 
        app_attrs: Union[ApplicationAttributes, Dict[str, Any]]
    ) -> None:
        """Register a new application."""
        if isinstance(app_attrs, dict):
            app_attrs = ApplicationAttributes(**app_attrs)
        
        app_spec = ApplicationSpec(
            shim=app_attrs.shim,
            url=app_attrs.url,
            command=app_attrs.command,
            arguments=app_attrs.arguments or [],
            environments=app_attrs.environments or [],
            working_directory=app_attrs.working_directory
        )
        
        request = RegisterApplicationRequest(
            name=name,
            application=app_spec
        )
        
        try:
            await self._frontend.RegisterApplication(request)
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to register application: {e.details()}"
            )
    
    async def unregister_application(self, name: str) -> None:
        """Unregister an application."""
        request = UnregisterApplicationRequest(name=name)
        
        try:
            await self._frontend.UnregisterApplication(request)
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to unregister application: {e.details()}"
            )
    
    async def list_applications(self) -> List[Application]:
        """List all applications."""
        request = ListApplicationRequest()
        
        try:
            response = await self._frontend.ListApplication(request)
            
            applications = []
            for app in response.applications:
                applications.append(Application(
                    id=app.metadata.id,
                    name=app.metadata.name,
                    shim=Shim(app.spec.shim),
                    state=ApplicationState(app.status.state),
                    creation_time=datetime.fromtimestamp(app.status.creation_time),
                    url=app.spec.url,
                    command=app.spec.command,
                    arguments=list(app.spec.arguments),
                    environments=list(app.spec.environments),
                    working_directory=app.spec.working_directory
                ))
            
            return applications
            
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to list applications: {e.details()}"
            )
    
    async def create_session(self, attrs: SessionAttributes) -> "Session":
        """Create a new session."""
        session_spec = SessionSpec(
            application=attrs.application,
            slots=attrs.slots,
            common_data=attrs.common_data
        )
        
        request = CreateSessionRequest(session=session_spec)
        
        try:
            response = await self._frontend.CreateSession(request)
            
            session = Session(
                connection=self,
                id=response.metadata.id,
                application=response.spec.application,
                slots=response.spec.slots,
                state=SessionState(response.status.state),
                creation_time=datetime.fromtimestamp(response.status.creation_time),
                pending=response.status.pending,
                running=response.status.running,
                succeed=response.status.succeed,
                failed=response.status.failed,
                completion_time=datetime.fromtimestamp(response.status.completion_time) if response.status.HasField('completion_time') else None
            )
            return session
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to create session: {e.details()}"
            )
    
    async def list_sessions(self) -> List["Session"]:
        """List all sessions."""
        request = ListSessionRequest()
        
        try:
            response = await self._frontend.ListSession(request)
            
            sessions = []
            for session in response.sessions:
                sessions.append(Session(
                    connection=self,
                    id=session.metadata.id,
                    application=session.spec.application,
                    slots=session.spec.slots,
                    state=SessionState(session.status.state),
                    creation_time=datetime.fromtimestamp(session.status.creation_time),
                    pending=session.status.pending,
                    running=session.status.running,
                    succeed=session.status.succeed,
                    failed=session.status.failed,
                    completion_time=datetime.fromtimestamp(session.status.completion_time) if session.status.HasField('completion_time') else None
                ))
            
            return sessions
            
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to list sessions: {e.details()}"
            )
    
    async def get_session(self, session_id: SessionID) -> "Session":
        """Get a session by ID."""
        request = GetSessionRequest(session_id=session_id)
        
        try:
            response = await self._frontend.GetSession(request)
            
            return Session(
                connection=self,
                id=response.metadata.id,
                application=response.spec.application,
                slots=response.spec.slots,
                state=SessionState(response.status.state),
                creation_time=datetime.fromtimestamp(response.status.creation_time),
                pending=response.status.pending,
                running=response.status.running,
                succeed=response.status.succeed,
                failed=response.status.failed,
                completion_time=datetime.fromtimestamp(response.status.completion_time) if response.status.HasField('completion_time') else None
            )
            
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to get session: {e.details()}"
            )
    
    async def close_session(self, session_id: SessionID) -> "Session":
        """Close a session."""
        request = CloseSessionRequest(session_id=session_id)
        
        try:
            response = await self._frontend.CloseSession(request)
            
            return Session(
                connection=self,
                id=response.metadata.id,
                application=response.spec.application,
                slots=response.spec.slots,
                state=SessionState(response.status.state),
                creation_time=datetime.fromtimestamp(response.status.creation_time),
                pending=response.status.pending,
                running=response.status.running,
                succeed=response.status.succeed,
                failed=response.status.failed,
                completion_time=datetime.fromtimestamp(response.status.completion_time) if response.status.HasField('completion_time') else None
            )
            
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to close session: {e.details()}"
            )

class Session:
    connection: Connection
    
    """Represents a computing session."""
    id: SessionID
    application: str
    slots: int
    state: SessionState
    creation_time: datetime
    pending: int = 0
    running: int = 0
    succeed: int = 0
    failed: int = 0
    completion_time: Optional[datetime] = None

    """Client for session-specific operations."""
    
    def __init__(self, connection: Connection, id: SessionID, application: str, slots: int, state: SessionState, creation_time: datetime, pending: int, running: int, succeed: int, failed: int, completion_time: Optional[datetime]):
        self.connection = connection
        self.id = id
        self.application = application
        self.slots = slots
        self.state = state
        self.creation_time = creation_time
        self.pending = pending
        self.running = running
        self.succeed = succeed
        self.failed = failed
        self.completion_time = completion_time

    async def create_task(self, input_data: TaskInput) -> Task:
        """Create a new task in the session."""
        task_spec = TaskSpec(
            session_id=self.id,
            input=input_data
        )
        
        request = CreateTaskRequest(task=task_spec)
        
        try:
            response = await self.connection._frontend.CreateTask(request)
            
            return Task(
                id=response.metadata.id,
                session_id=self.id,
                state=TaskState(response.status.state),
                creation_time=datetime.fromtimestamp(response.status.creation_time),
                input=input_data,
                completion_time=datetime.fromtimestamp(response.status.completion_time) if response.status.HasField('completion_time') else None
            )
            
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to create task: {e.details()}"
            )
    
    async def get_task(self, task_id: TaskID) -> Task:
        """Get a task by ID."""
        request = GetTaskRequest(
            task_id=task_id,
            session_id=self.id
        )
        
        try:
            response = await self.connection._frontend.GetTask(request)
            
            return Task(
                id=response.metadata.id,
                session_id=self.id,
                state=TaskState(response.status.state),
                creation_time=datetime.fromtimestamp(response.status.creation_time),
                input=response.spec.input,
                output=response.spec.output,
                completion_time=datetime.fromtimestamp(response.status.completion_time) if response.status.HasField('completion_time') else None
            )
            
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to get task: {e.details()}"
            )
    
    async def watch_task(self, task_id: TaskID) -> "TaskWatcher":
        """Watch a task for updates."""
        request = WatchTaskRequest(
            task_id=task_id,
            session_id=self.id
        )
        
        try:
            stream = self.connection._frontend.WatchTask(request)
            return TaskWatcher(stream)
            
        except grpc.RpcError as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to watch task: {e.details()}"
            )
    
    async def invoke(self, input_data: TaskInput, informer: Optional[TaskInformer] = None) -> Task:
        """Invoke a task with the given input and optional informer."""
        task = await self.create_task(input_data)
        watcher = await self.watch_task(task.id)
        
        async for task in watcher:
            if informer:
                informer.on_update(task)
            if task.is_completed():
                return task

    async def close(self) -> None:
        """Close the session."""
        await self.connection.close_session(self.id)

class TaskWatcher:
    """Async iterator for watching task updates."""
    
    def __init__(self, stream):
        self._stream = stream
    
    def __aiter__(self):
        return self
    
    async def __anext__(self) -> Task:
        try:
            response = await self._stream.read()
            
            return Task(
                id=response.metadata.id,
                session_id=response.spec.session_id,
                state=TaskState(response.status.state),
                creation_time=datetime.fromtimestamp(response.status.creation_time),
                input=response.spec.input,
                output=response.spec.output,
                completion_time=datetime.fromtimestamp(response.status.completion_time) if response.status.HasField('completion_time') else None
            )
            
        except StopAsyncIteration:
            raise
        except Exception as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"failed to watch task: {str(e)}"
            ) 
