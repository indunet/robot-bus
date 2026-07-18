from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import quaternion_pb2 as _quaternion_pb2
from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Imu(_message.Message):
    __slots__ = ("header", "orientation", "orientation_covariance", "angular_velocity", "angular_velocity_covariance", "linear_acceleration", "linear_acceleration_covariance")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    ORIENTATION_FIELD_NUMBER: _ClassVar[int]
    ORIENTATION_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    ANGULAR_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    ANGULAR_VELOCITY_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    LINEAR_ACCELERATION_FIELD_NUMBER: _ClassVar[int]
    LINEAR_ACCELERATION_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    orientation: _quaternion_pb2.Quaternion
    orientation_covariance: _containers.RepeatedScalarFieldContainer[float]
    angular_velocity: _vector3_pb2.Vector3
    angular_velocity_covariance: _containers.RepeatedScalarFieldContainer[float]
    linear_acceleration: _vector3_pb2.Vector3
    linear_acceleration_covariance: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., orientation: _Optional[_Union[_quaternion_pb2.Quaternion, _Mapping]] = ..., orientation_covariance: _Optional[_Iterable[float]] = ..., angular_velocity: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., angular_velocity_covariance: _Optional[_Iterable[float]] = ..., linear_acceleration: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., linear_acceleration_covariance: _Optional[_Iterable[float]] = ...) -> None: ...
