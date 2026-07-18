from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import point32_pb2 as _point32_pb2
from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class VoxelGrid(_message.Message):
    __slots__ = ("header", "data", "origin", "resolutions", "size_x", "size_y", "size_z")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    RESOLUTIONS_FIELD_NUMBER: _ClassVar[int]
    SIZE_X_FIELD_NUMBER: _ClassVar[int]
    SIZE_Y_FIELD_NUMBER: _ClassVar[int]
    SIZE_Z_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    data: _containers.RepeatedScalarFieldContainer[int]
    origin: _point32_pb2.Point32
    resolutions: _vector3_pb2.Vector3
    size_x: int
    size_y: int
    size_z: int
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., data: _Optional[_Iterable[int]] = ..., origin: _Optional[_Union[_point32_pb2.Point32, _Mapping]] = ..., resolutions: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., size_x: _Optional[int] = ..., size_y: _Optional[int] = ..., size_z: _Optional[int] = ...) -> None: ...
