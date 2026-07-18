from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import point_pb2 as _point_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class GridCells(_message.Message):
    __slots__ = ("header", "cell_width", "cell_height", "cells")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    CELL_WIDTH_FIELD_NUMBER: _ClassVar[int]
    CELL_HEIGHT_FIELD_NUMBER: _ClassVar[int]
    CELLS_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    cell_width: float
    cell_height: float
    cells: _containers.RepeatedCompositeFieldContainer[_point_pb2.Point]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., cell_width: _Optional[float] = ..., cell_height: _Optional[float] = ..., cells: _Optional[_Iterable[_Union[_point_pb2.Point, _Mapping]]] = ...) -> None: ...
