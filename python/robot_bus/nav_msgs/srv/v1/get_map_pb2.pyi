from robot_bus.nav_msgs.msg.v1 import occupancy_grid_pb2 as _occupancy_grid_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class GetMapRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class GetMapResponse(_message.Message):
    __slots__ = ("map",)
    MAP_FIELD_NUMBER: _ClassVar[int]
    map: _occupancy_grid_pb2.OccupancyGrid
    def __init__(self, map: _Optional[_Union[_occupancy_grid_pb2.OccupancyGrid, _Mapping]] = ...) -> None: ...
