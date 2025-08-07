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
import os
import grpc
from abc import ABC, abstractmethod
from typing import Optional, Dict, Any, Union
from dataclasses import dataclass

from .types import Shim, FlameError, FlameErrorCode
from .protos import placeholder as shim_pb2
from .protos import placeholder as types_pb2


@dataclass
class ApplicationContext:
    """Context for an application."""
    name: str
    shim: Shim
    url: Optional[str] = None
    command: Optional[str] = None


@dataclass
class SessionContext:
    """Context for a session."""
    session_id: str
    application: ApplicationContext
    common_data: Optional[bytes] = None


@dataclass
class TaskContext:
    """Context for a task."""
    task_id: str
    session_id: str
    input: Optional[bytes] = None


@dataclass
class TaskOutput:
    """Output from a task."""
    data: Optional[bytes] = None


class FlameService:
    """Base class for implementing Flame services."""
    
    @abstractmethod
    async def on_session_enter(self, context: SessionContext) -> bool:
        """
        Called when entering a session.
        
        Args:
            context: Session context information
            
        Returns:
            True if successful, False otherwise
        """
        pass
    
    @abstractmethod
    async def on_task_invoke(self, context: TaskContext) -> TaskOutput:
        """
        Called when a task is invoked.
        
        Args:
            context: Task context information
            
        Returns:
            Task output
        """
        pass
    
    @abstractmethod
    async def on_session_leave(self) -> bool:
        """
        Called when leaving a session.
        
        Returns:
            True if successful, False otherwise
        """
        pass


class GrpcShimServicer(shim_pb2.GrpcShimServicer):
    """gRPC servicer implementation for GrpcShim service."""
    
    def __init__(self, service: FlameService):
        self._service = service
    
    async def OnSessionEnter(self, request, context):
        """Handle OnSessionEnter RPC call."""
        try:
            # Convert protobuf request to SessionContext
            app_context = ApplicationContext(
                name=request.application.name,
                shim=Shim(request.application.shim),
                url=request.application.url,
                command=request.application.command
            )
            
            session_context = SessionContext(
                session_id=request.session_id,
                application=app_context,
                common_data=request.common_data
            )
            
            # Call the service implementation
            success = await self._service.on_session_enter(session_context)
            
            # Return result
            return types_pb2.Result(
                return_code=0 if success else 1,
                message="Session enter successful" if success else "Session enter failed"
            )
            
        except Exception as e:
            return types_pb2.Result(
                return_code=1,
                message=f"Session enter error: {str(e)}"
            )
    
    async def OnTaskInvoke(self, request, context):
        """Handle OnTaskInvoke RPC call."""
        try:
            # Convert protobuf request to TaskContext
            task_context = TaskContext(
                task_id=request.task_id,
                session_id=request.session_id,
                input=request.input
            )
            
            # Call the service implementation
            output = await self._service.on_task_invoke(task_context)
            
            # Return task output
            return shim_pb2.TaskOutput(data=output.data)
            
        except Exception as e:
            return shim_pb2.TaskOutput(data=None)
    
    async def OnSessionLeave(self, request, context):
        """Handle OnSessionLeave RPC call."""
        try:
            # Call the service implementation
            success = await self._service.on_session_leave()
            
            # Return result
            return types_pb2.Result(
                return_code=0 if success else 1,
                message="Session leave successful" if success else "Session leave failed"
            )
            
        except Exception as e:
            return types_pb2.Result(
                return_code=1,
                message=f"Session leave error: {str(e)}"
            )

class GrpcShimServer:
    """Server for gRPC shim services."""
    
    def __init__(self, service: FlameService):
        self._service = service
        self._server = None
    
    async def start(self):
        """Start the gRPC server."""
        try:
            # Create gRPC server
            self._server = grpc.aio.server()
            
            # Add servicer to server
            shim_servicer = GrpcShimServicer(self._service)
            shim_pb2.add_GrpcShimServicer_to_server(shim_servicer, self._server)
            
            # Listen on Unix socket
            socket_path = f"/tmp/flame/shim/{os.getpid()}.sock"
            self._server.add_insecure_port(f"unix://{socket_path}")
            
            # Start server
            await self._server.start()
            
            print(f"Flame Python service started on Unix socket: {socket_path}")

            # Keep server running
            await self._server.wait_for_termination()
            
        except Exception as e:
            raise FlameError(
                FlameErrorCode.INTERNAL,
                f"Failed to start gRPC server: {str(e)}"
            )
    
    async def stop(self):
        """Stop the gRPC server."""
        if self._server:
            await self._server.stop(grace=5)
            print("gRPC shim server stopped")


async def run(service: FlameService):
    """
    Run a gRPC shim server.
    
    Args:
        service: The shim service implementation
    """

    server = GrpcShimServer(service)
    await server.start()

