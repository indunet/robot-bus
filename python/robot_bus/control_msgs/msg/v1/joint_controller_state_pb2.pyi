from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class JointControllerState(_message.Message):
    __slots__ = ("header", "set_point", "process_value", "process_value_dot", "error", "time_step", "command", "p", "i", "d", "i_clamp", "antiwindup")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    SET_POINT_FIELD_NUMBER: _ClassVar[int]
    PROCESS_VALUE_FIELD_NUMBER: _ClassVar[int]
    PROCESS_VALUE_DOT_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    TIME_STEP_FIELD_NUMBER: _ClassVar[int]
    COMMAND_FIELD_NUMBER: _ClassVar[int]
    P_FIELD_NUMBER: _ClassVar[int]
    I_FIELD_NUMBER: _ClassVar[int]
    D_FIELD_NUMBER: _ClassVar[int]
    I_CLAMP_FIELD_NUMBER: _ClassVar[int]
    ANTIWINDUP_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    set_point: float
    process_value: float
    process_value_dot: float
    error: float
    time_step: float
    command: float
    p: float
    i: float
    d: float
    i_clamp: float
    antiwindup: bool
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., set_point: _Optional[float] = ..., process_value: _Optional[float] = ..., process_value_dot: _Optional[float] = ..., error: _Optional[float] = ..., time_step: _Optional[float] = ..., command: _Optional[float] = ..., p: _Optional[float] = ..., i: _Optional[float] = ..., d: _Optional[float] = ..., i_clamp: _Optional[float] = ..., antiwindup: bool = ...) -> None: ...
