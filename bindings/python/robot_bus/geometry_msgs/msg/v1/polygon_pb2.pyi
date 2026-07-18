from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import point32_pb2 as _point32_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Polygon(_message.Message):
    __slots__ = ("points",)
    POINTS_FIELD_NUMBER: _ClassVar[int]
    points: _containers.RepeatedCompositeFieldContainer[_point32_pb2.Point32]
    def __init__(self, points: _Optional[_Iterable[_Union[_point32_pb2.Point32, _Mapping]]] = ...) -> None: ...

class PolygonStamped(_message.Message):
    __slots__ = ("header", "polygon")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POLYGON_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    polygon: Polygon
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., polygon: _Optional[_Union[Polygon, _Mapping]] = ...) -> None: ...
