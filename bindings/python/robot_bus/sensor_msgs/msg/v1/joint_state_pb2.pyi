from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class JointState(_message.Message):
    __slots__ = ("header", "name", "position", "velocity", "effort")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VELOCITY_FIELD_NUMBER: _ClassVar[int]
    EFFORT_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    name: _containers.RepeatedScalarFieldContainer[str]
    position: _containers.RepeatedScalarFieldContainer[float]
    velocity: _containers.RepeatedScalarFieldContainer[float]
    effort: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., name: _Optional[_Iterable[str]] = ..., position: _Optional[_Iterable[float]] = ..., velocity: _Optional[_Iterable[float]] = ..., effort: _Optional[_Iterable[float]] = ...) -> None: ...
