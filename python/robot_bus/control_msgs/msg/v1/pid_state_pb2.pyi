from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.builtin_interfaces.msg.v1 import duration_pb2 as _duration_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PidState(_message.Message):
    __slots__ = ("header", "timestep", "error", "error_dot", "p_error", "i_error", "d_error", "p_term", "i_term", "d_term", "i_max", "i_min", "output")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    TIMESTEP_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    ERROR_DOT_FIELD_NUMBER: _ClassVar[int]
    P_ERROR_FIELD_NUMBER: _ClassVar[int]
    I_ERROR_FIELD_NUMBER: _ClassVar[int]
    D_ERROR_FIELD_NUMBER: _ClassVar[int]
    P_TERM_FIELD_NUMBER: _ClassVar[int]
    I_TERM_FIELD_NUMBER: _ClassVar[int]
    D_TERM_FIELD_NUMBER: _ClassVar[int]
    I_MAX_FIELD_NUMBER: _ClassVar[int]
    I_MIN_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    timestep: _duration_pb2.Duration
    error: float
    error_dot: float
    p_error: float
    i_error: float
    d_error: float
    p_term: float
    i_term: float
    d_term: float
    i_max: float
    i_min: float
    output: float
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., timestep: _Optional[_Union[_duration_pb2.Duration, _Mapping]] = ..., error: _Optional[float] = ..., error_dot: _Optional[float] = ..., p_error: _Optional[float] = ..., i_error: _Optional[float] = ..., d_error: _Optional[float] = ..., p_term: _Optional[float] = ..., i_term: _Optional[float] = ..., d_term: _Optional[float] = ..., i_max: _Optional[float] = ..., i_min: _Optional[float] = ..., output: _Optional[float] = ...) -> None: ...
