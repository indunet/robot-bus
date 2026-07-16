from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class JointJog(_message.Message):
    __slots__ = ("header", "joint_names", "displacements", "velocities", "duration")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    DISPLACEMENTS_FIELD_NUMBER: _ClassVar[int]
    VELOCITIES_FIELD_NUMBER: _ClassVar[int]
    DURATION_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    joint_names: _containers.RepeatedScalarFieldContainer[str]
    displacements: _containers.RepeatedScalarFieldContainer[float]
    velocities: _containers.RepeatedScalarFieldContainer[float]
    duration: float
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., joint_names: _Optional[_Iterable[str]] = ..., displacements: _Optional[_Iterable[float]] = ..., velocities: _Optional[_Iterable[float]] = ..., duration: _Optional[float] = ...) -> None: ...
