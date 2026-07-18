from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.builtin_interfaces.msg.v1 import duration_pb2 as _duration_pb2
from robot_bus.geometry_msgs.msg.v1 import transform_pb2 as _transform_pb2
from robot_bus.geometry_msgs.msg.v1 import twist_pb2 as _twist_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MultiDOFJointTrajectoryPoint(_message.Message):
    __slots__ = ("transforms", "velocities", "accelerations", "time_from_start")
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    VELOCITIES_FIELD_NUMBER: _ClassVar[int]
    ACCELERATIONS_FIELD_NUMBER: _ClassVar[int]
    TIME_FROM_START_FIELD_NUMBER: _ClassVar[int]
    transforms: _containers.RepeatedCompositeFieldContainer[_transform_pb2.Transform]
    velocities: _containers.RepeatedCompositeFieldContainer[_twist_pb2.Twist]
    accelerations: _containers.RepeatedCompositeFieldContainer[_twist_pb2.Twist]
    time_from_start: _duration_pb2.Duration
    def __init__(self, transforms: _Optional[_Iterable[_Union[_transform_pb2.Transform, _Mapping]]] = ..., velocities: _Optional[_Iterable[_Union[_twist_pb2.Twist, _Mapping]]] = ..., accelerations: _Optional[_Iterable[_Union[_twist_pb2.Twist, _Mapping]]] = ..., time_from_start: _Optional[_Union[_duration_pb2.Duration, _Mapping]] = ...) -> None: ...

class MultiDOFJointTrajectory(_message.Message):
    __slots__ = ("header", "joint_names", "points")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    joint_names: _containers.RepeatedScalarFieldContainer[str]
    points: _containers.RepeatedCompositeFieldContainer[MultiDOFJointTrajectoryPoint]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., joint_names: _Optional[_Iterable[str]] = ..., points: _Optional[_Iterable[_Union[MultiDOFJointTrajectoryPoint, _Mapping]]] = ...) -> None: ...
