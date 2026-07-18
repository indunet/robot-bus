from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MagneticField(_message.Message):
    __slots__ = ("header", "magnetic_field", "magnetic_field_covariance")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    MAGNETIC_FIELD_FIELD_NUMBER: _ClassVar[int]
    MAGNETIC_FIELD_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    magnetic_field: _vector3_pb2.Vector3
    magnetic_field_covariance: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., magnetic_field: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., magnetic_field_covariance: _Optional[_Iterable[float]] = ...) -> None: ...
