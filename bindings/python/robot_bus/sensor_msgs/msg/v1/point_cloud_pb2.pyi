from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import point32_pb2 as _point32_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ChannelFloat32(_message.Message):
    __slots__ = ("name", "values")
    NAME_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    name: str
    values: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, name: _Optional[str] = ..., values: _Optional[_Iterable[float]] = ...) -> None: ...

class PointCloud(_message.Message):
    __slots__ = ("header", "points", "channels")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    CHANNELS_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    points: _containers.RepeatedCompositeFieldContainer[_point32_pb2.Point32]
    channels: _containers.RepeatedCompositeFieldContainer[ChannelFloat32]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., points: _Optional[_Iterable[_Union[_point32_pb2.Point32, _Mapping]]] = ..., channels: _Optional[_Iterable[_Union[ChannelFloat32, _Mapping]]] = ...) -> None: ...
