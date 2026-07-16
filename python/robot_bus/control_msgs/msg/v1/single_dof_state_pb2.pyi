from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SingleDOFState(_message.Message):
    __slots__ = ("name", "reference", "feedback", "feedback_dot", "error", "error_dot", "time_step", "output")
    NAME_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    FEEDBACK_FIELD_NUMBER: _ClassVar[int]
    FEEDBACK_DOT_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    ERROR_DOT_FIELD_NUMBER: _ClassVar[int]
    TIME_STEP_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    name: str
    reference: float
    feedback: float
    feedback_dot: float
    error: float
    error_dot: float
    time_step: float
    output: float
    def __init__(self, name: _Optional[str] = ..., reference: _Optional[float] = ..., feedback: _Optional[float] = ..., feedback_dot: _Optional[float] = ..., error: _Optional[float] = ..., error_dot: _Optional[float] = ..., time_step: _Optional[float] = ..., output: _Optional[float] = ...) -> None: ...

class SingleDOFStateStamped(_message.Message):
    __slots__ = ("header", "state")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    state: SingleDOFState
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., state: _Optional[_Union[SingleDOFState, _Mapping]] = ...) -> None: ...
