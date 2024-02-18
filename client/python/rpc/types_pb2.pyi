from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SessionState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    SessionOpen: _ClassVar[SessionState]
    SessionClosed: _ClassVar[SessionState]

class TaskState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    TaskPending: _ClassVar[TaskState]
    TaskRunning: _ClassVar[TaskState]
    TaskSucceed: _ClassVar[TaskState]
    TaskFailed: _ClassVar[TaskState]

class Shim(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    LogShim: _ClassVar[Shim]
    StdioShim: _ClassVar[Shim]
    RpcShim: _ClassVar[Shim]
    RestShim: _ClassVar[Shim]

class ExecutorState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    ExecutorIdle: _ClassVar[ExecutorState]
    ExecutorBound: _ClassVar[ExecutorState]
    ExecutorRunning: _ClassVar[ExecutorState]
    ExecutorUnknown: _ClassVar[ExecutorState]
SessionOpen: SessionState
SessionClosed: SessionState
TaskPending: TaskState
TaskRunning: TaskState
TaskSucceed: TaskState
TaskFailed: TaskState
LogShim: Shim
StdioShim: Shim
RpcShim: Shim
RestShim: Shim
ExecutorIdle: ExecutorState
ExecutorBound: ExecutorState
ExecutorRunning: ExecutorState
ExecutorUnknown: ExecutorState

class Metadata(_message.Message):
    __slots__ = ("id", "owner")
    ID_FIELD_NUMBER: _ClassVar[int]
    OWNER_FIELD_NUMBER: _ClassVar[int]
    id: str
    owner: str
    def __init__(self, id: _Optional[str] = ..., owner: _Optional[str] = ...) -> None: ...

class SessionStatus(_message.Message):
    __slots__ = ("state", "creation_time", "completion_time", "pending", "running", "succeed", "failed")
    STATE_FIELD_NUMBER: _ClassVar[int]
    CREATION_TIME_FIELD_NUMBER: _ClassVar[int]
    COMPLETION_TIME_FIELD_NUMBER: _ClassVar[int]
    PENDING_FIELD_NUMBER: _ClassVar[int]
    RUNNING_FIELD_NUMBER: _ClassVar[int]
    SUCCEED_FIELD_NUMBER: _ClassVar[int]
    FAILED_FIELD_NUMBER: _ClassVar[int]
    state: SessionState
    creation_time: int
    completion_time: int
    pending: int
    running: int
    succeed: int
    failed: int
    def __init__(self, state: _Optional[_Union[SessionState, str]] = ..., creation_time: _Optional[int] = ..., completion_time: _Optional[int] = ..., pending: _Optional[int] = ..., running: _Optional[int] = ..., succeed: _Optional[int] = ..., failed: _Optional[int] = ...) -> None: ...

class SessionSpec(_message.Message):
    __slots__ = ("application", "slots", "common_data")
    APPLICATION_FIELD_NUMBER: _ClassVar[int]
    SLOTS_FIELD_NUMBER: _ClassVar[int]
    COMMON_DATA_FIELD_NUMBER: _ClassVar[int]
    application: str
    slots: int
    common_data: bytes
    def __init__(self, application: _Optional[str] = ..., slots: _Optional[int] = ..., common_data: _Optional[bytes] = ...) -> None: ...

class Session(_message.Message):
    __slots__ = ("metadata", "spec", "status")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SPEC_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    metadata: Metadata
    spec: SessionSpec
    status: SessionStatus
    def __init__(self, metadata: _Optional[_Union[Metadata, _Mapping]] = ..., spec: _Optional[_Union[SessionSpec, _Mapping]] = ..., status: _Optional[_Union[SessionStatus, _Mapping]] = ...) -> None: ...

class TaskStatus(_message.Message):
    __slots__ = ("state", "creation_time", "completion_time")
    STATE_FIELD_NUMBER: _ClassVar[int]
    CREATION_TIME_FIELD_NUMBER: _ClassVar[int]
    COMPLETION_TIME_FIELD_NUMBER: _ClassVar[int]
    state: TaskState
    creation_time: int
    completion_time: int
    def __init__(self, state: _Optional[_Union[TaskState, str]] = ..., creation_time: _Optional[int] = ..., completion_time: _Optional[int] = ...) -> None: ...

class TaskSpec(_message.Message):
    __slots__ = ("session_id", "input", "output")
    SESSION_ID_FIELD_NUMBER: _ClassVar[int]
    INPUT_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    session_id: str
    input: bytes
    output: bytes
    def __init__(self, session_id: _Optional[str] = ..., input: _Optional[bytes] = ..., output: _Optional[bytes] = ...) -> None: ...

class Task(_message.Message):
    __slots__ = ("metadata", "spec", "status")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SPEC_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    metadata: Metadata
    spec: TaskSpec
    status: TaskStatus
    def __init__(self, metadata: _Optional[_Union[Metadata, _Mapping]] = ..., spec: _Optional[_Union[TaskSpec, _Mapping]] = ..., status: _Optional[_Union[TaskStatus, _Mapping]] = ...) -> None: ...

class Application(_message.Message):
    __slots__ = ("name", "shim", "command", "arguments", "environments", "working_directory")
    NAME_FIELD_NUMBER: _ClassVar[int]
    SHIM_FIELD_NUMBER: _ClassVar[int]
    COMMAND_FIELD_NUMBER: _ClassVar[int]
    ARGUMENTS_FIELD_NUMBER: _ClassVar[int]
    ENVIRONMENTS_FIELD_NUMBER: _ClassVar[int]
    WORKING_DIRECTORY_FIELD_NUMBER: _ClassVar[int]
    name: str
    shim: Shim
    command: str
    arguments: _containers.RepeatedScalarFieldContainer[str]
    environments: _containers.RepeatedScalarFieldContainer[str]
    working_directory: str
    def __init__(self, name: _Optional[str] = ..., shim: _Optional[_Union[Shim, str]] = ..., command: _Optional[str] = ..., arguments: _Optional[_Iterable[str]] = ..., environments: _Optional[_Iterable[str]] = ..., working_directory: _Optional[str] = ...) -> None: ...

class ExecutorSpec(_message.Message):
    __slots__ = ("slots", "applications")
    SLOTS_FIELD_NUMBER: _ClassVar[int]
    APPLICATIONS_FIELD_NUMBER: _ClassVar[int]
    slots: int
    applications: _containers.RepeatedCompositeFieldContainer[Application]
    def __init__(self, slots: _Optional[int] = ..., applications: _Optional[_Iterable[_Union[Application, _Mapping]]] = ...) -> None: ...

class ExecutorStatus(_message.Message):
    __slots__ = ("state",)
    STATE_FIELD_NUMBER: _ClassVar[int]
    state: ExecutorState
    def __init__(self, state: _Optional[_Union[ExecutorState, str]] = ...) -> None: ...

class Executor(_message.Message):
    __slots__ = ("metadata", "spec", "status")
    METADATA_FIELD_NUMBER: _ClassVar[int]
    SPEC_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    metadata: Metadata
    spec: ExecutorSpec
    status: ExecutorStatus
    def __init__(self, metadata: _Optional[_Union[Metadata, _Mapping]] = ..., spec: _Optional[_Union[ExecutorSpec, _Mapping]] = ..., status: _Optional[_Union[ExecutorStatus, _Mapping]] = ...) -> None: ...

class Result(_message.Message):
    __slots__ = ("return_code", "message")
    RETURN_CODE_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    return_code: int
    message: str
    def __init__(self, return_code: _Optional[int] = ..., message: _Optional[str] = ...) -> None: ...

class SessionList(_message.Message):
    __slots__ = ("sessions",)
    SESSIONS_FIELD_NUMBER: _ClassVar[int]
    sessions: _containers.RepeatedCompositeFieldContainer[Session]
    def __init__(self, sessions: _Optional[_Iterable[_Union[Session, _Mapping]]] = ...) -> None: ...
