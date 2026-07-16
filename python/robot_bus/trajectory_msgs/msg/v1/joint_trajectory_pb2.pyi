from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.builtin_interfaces.msg.v1 import duration_pb2 as _duration_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class JointTrajectoryPoint(_message.Message):
    __slots__ = ("positions", "velocities", "accelerations", "effort", "time_from_start")
    POSITIONS_FIELD_NUMBER: _ClassVar[int]
    VELOCITIES_FIELD_NUMBER: _ClassVar[int]
    ACCELERATIONS_FIELD_NUMBER: _ClassVar[int]
    EFFORT_FIELD_NUMBER: _ClassVar[int]
    TIME_FROM_START_FIELD_NUMBER: _ClassVar[int]
    positions: _containers.RepeatedScalarFieldContainer[float]
    velocities: _containers.RepeatedScalarFieldContainer[float]
    accelerations: _containers.RepeatedScalarFieldContainer[float]
    effort: _containers.RepeatedScalarFieldContainer[float]
    time_from_start: _duration_pb2.Duration
    def __init__(self, positions: _Optional[_Iterable[float]] = ..., velocities: _Optional[_Iterable[float]] = ..., accelerations: _Optional[_Iterable[float]] = ..., effort: _Optional[_Iterable[float]] = ..., time_from_start: _Optional[_Union[_duration_pb2.Duration, _Mapping]] = ...) -> None: ...

class JointTrajectory(_message.Message):
    __slots__ = ("header", "joint_names", "points")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    joint_names: _containers.RepeatedScalarFieldContainer[str]
    points: _containers.RepeatedCompositeFieldContainer[JointTrajectoryPoint]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., joint_names: _Optional[_Iterable[str]] = ..., points: _Optional[_Iterable[_Union[JointTrajectoryPoint, _Mapping]]] = ...) -> None: ...
