from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.builtin_interfaces.msg.v1 import time_pb2 as _time_pb2
from robot_bus.geometry_msgs.msg.v1 import pose_pb2 as _pose_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MapMetaData(_message.Message):
    __slots__ = ("map_load_time", "resolution", "width", "height", "origin")
    MAP_LOAD_TIME_FIELD_NUMBER: _ClassVar[int]
    RESOLUTION_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    map_load_time: _time_pb2.Time
    resolution: float
    width: int
    height: int
    origin: _pose_pb2.Pose
    def __init__(self, map_load_time: _Optional[_Union[_time_pb2.Time, _Mapping]] = ..., resolution: _Optional[float] = ..., width: _Optional[int] = ..., height: _Optional[int] = ..., origin: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ...) -> None: ...

class OccupancyGrid(_message.Message):
    __slots__ = ("header", "info", "data")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    INFO_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    info: MapMetaData
    data: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., info: _Optional[_Union[MapMetaData, _Mapping]] = ..., data: _Optional[_Iterable[int]] = ...) -> None: ...
