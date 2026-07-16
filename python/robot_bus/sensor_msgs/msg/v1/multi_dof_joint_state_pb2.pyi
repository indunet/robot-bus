from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import transform_pb2 as _transform_pb2
from robot_bus.geometry_msgs.msg.v1 import twist_pb2 as _twist_pb2
from robot_bus.geometry_msgs.msg.v1 import accel_wrench_pb2 as _accel_wrench_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MultiDOFJointState(_message.Message):
    __slots__ = ("header", "joint_names", "transforms", "twist", "wrench")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    TWIST_FIELD_NUMBER: _ClassVar[int]
    WRENCH_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    joint_names: _containers.RepeatedScalarFieldContainer[str]
    transforms: _containers.RepeatedCompositeFieldContainer[_transform_pb2.Transform]
    twist: _containers.RepeatedCompositeFieldContainer[_twist_pb2.Twist]
    wrench: _containers.RepeatedCompositeFieldContainer[_accel_wrench_pb2.Wrench]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., joint_names: _Optional[_Iterable[str]] = ..., transforms: _Optional[_Iterable[_Union[_transform_pb2.Transform, _Mapping]]] = ..., twist: _Optional[_Iterable[_Union[_twist_pb2.Twist, _Mapping]]] = ..., wrench: _Optional[_Iterable[_Union[_accel_wrench_pb2.Wrench, _Mapping]]] = ...) -> None: ...
