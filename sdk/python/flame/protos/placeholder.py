"""
Placeholder protobuf files for development.
These will be replaced by actual generated protobuf files.
"""

# Placeholder classes for development
class FrontendStub:
    """Placeholder for FrontendStub."""
    pass

class TaskWatcher:
    """Placeholder for TaskWatcher."""
    pass

# Shim service classes
class GrpcShimServicer:
    """Placeholder for GrpcShimServicer."""
    pass

class GrpcServiceManagerServicer:
    """Placeholder for GrpcServiceManagerServicer."""
    pass

def add_GrpcShimServicer_to_server(servicer, server):
    """Placeholder for add_GrpcShimServicer_to_server."""
    pass

def add_GrpcServiceManagerServicer_to_server(servicer, server):
    """Placeholder for add_GrpcServiceManagerServicer_to_server."""
    pass

# Placeholder message classes
class RegisterApplicationRequest:
    def __init__(self, name=None, application=None):
        self.name = name
        self.application = application

class UnregisterApplicationRequest:
    def __init__(self, name=None):
        self.name = name

class ListApplicationRequest:
    pass

class CreateSessionRequest:
    def __init__(self, session=None):
        self.session = session

class ListSessionRequest:
    pass

class GetSessionRequest:
    def __init__(self, session_id=None):
        self.session_id = session_id

class CloseSessionRequest:
    def __init__(self, session_id=None):
        self.session_id = session_id

class CreateTaskRequest:
    def __init__(self, task=None):
        self.task = task

class GetTaskRequest:
    def __init__(self, task_id=None, session_id=None):
        self.task_id = task_id
        self.session_id = session_id

class WatchTaskRequest:
    def __init__(self, task_id=None, session_id=None):
        self.task_id = task_id
        self.session_id = session_id

# Shim service message classes
class ApplicationContext:
    def __init__(self, name=None, shim=None, url=None, command=None, **kwargs):
        self.name = name
        self.shim = shim
        self.url = url
        self.command = command

class SessionContext:
    def __init__(self, session_id=None, application=None, common_data=None, **kwargs):
        self.session_id = session_id
        self.application = application
        self.common_data = common_data

class TaskContext:
    def __init__(self, task_id=None, session_id=None, input=None, **kwargs):
        self.task_id = task_id
        self.session_id = session_id
        self.input = input

class TaskOutput:
    def __init__(self, data=None, **kwargs):
        self.data = data

class RegisterServiceRequest:
    def __init__(self, address=None, service_id=None):
        self.address = address
        self.service_id = service_id

class RegisterServiceResponse:
    pass

# Placeholder for types
class ApplicationSpec:
    def __init__(self, shim=None, url=None, command=None, arguments=None, 
                 environments=None, working_directory=None):
        self.shim = shim
        self.url = url
        self.command = command
        self.arguments = arguments or []
        self.environments = environments or []
        self.working_directory = working_directory

class SessionSpec:
    def __init__(self, application=None, slots=None, common_data=None):
        self.application = application
        self.slots = slots
        self.common_data = common_data

class TaskSpec:
    def __init__(self, session_id=None, input=None, output=None):
        self.session_id = session_id
        self.input = input
        self.output = output

# Placeholder for response classes
class Application:
    def __init__(self, metadata=None, spec=None, status=None):
        self.metadata = metadata
        self.spec = spec
        self.status = status

class ApplicationList:
    def __init__(self, applications=None):
        self.applications = applications or []

class Session:
    def __init__(self, metadata=None, spec=None, status=None):
        self.metadata = metadata
        self.spec = spec
        self.status = status

class SessionList:
    def __init__(self, sessions=None):
        self.sessions = sessions or []

class Task:
    def __init__(self, metadata=None, spec=None, status=None):
        self.metadata = metadata
        self.spec = spec
        self.status = status

# Metadata classes
class Metadata:
    def __init__(self, id=None, name=None, owner=None):
        self.id = id
        self.name = name
        self.owner = owner

class SessionStatus:
    def __init__(self, state=None, creation_time=None, completion_time=None,
                 pending=None, running=None, succeed=None, failed=None):
        self.state = state
        self.creation_time = creation_time
        self.completion_time = completion_time
        self.pending = pending
        self.running = running
        self.succeed = succeed
        self.failed = failed

class TaskStatus:
    def __init__(self, state=None, creation_time=None, completion_time=None):
        self.state = state
        self.creation_time = creation_time
        self.completion_time = completion_time

class ApplicationStatus:
    def __init__(self, state=None, creation_time=None):
        self.state = state
        self.creation_time = creation_time

# Enums
class SessionState:
    OPEN = 0
    CLOSED = 1

class TaskState:
    PENDING = 0
    RUNNING = 1
    SUCCEED = 2
    FAILED = 3

class ApplicationState:
    ENABLED = 0
    DISABLED = 1

class Shim:
    LOG = 0
    STDIO = 1
    WASM = 2
    SHELL = 3
    GRPC = 4 