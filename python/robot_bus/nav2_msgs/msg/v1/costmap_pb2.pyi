from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.builtin_interfaces.msg.v1 import time_pb2 as _time_pb2
from robot_bus.geometry_msgs.msg.v1 import pose_pb2 as _pose_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CostmapMetaData(_message.Message):
    __slots__ = ("map_load_time", "update_time", "resolution", "size_x", "size_y", "origin", "layer")
    MAP_LOAD_TIME_FIELD_NUMBER: _ClassVar[int]
    UPDATE_TIME_FIELD_NUMBER: _ClassVar[int]
    RESOLUTION_FIELD_NUMBER: _ClassVar[int]
    SIZE_X_FIELD_NUMBER: _ClassVar[int]
    SIZE_Y_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_FIELD_NUMBER: _ClassVar[int]
    LAYER_FIELD_NUMBER: _ClassVar[int]
    map_load_time: _time_pb2.Time
    update_time: _time_pb2.Time
    resolution: float
    size_x: int
    size_y: int
    origin: _pose_pb2.Pose
    layer: str
    def __init__(self, map_load_time: _Optional[_Union[_time_pb2.Time, _Mapping]] = ..., update_time: _Optional[_Union[_time_pb2.Time, _Mapping]] = ..., resolution: _Optional[float] = ..., size_x: _Optional[int] = ..., size_y: _Optional[int] = ..., origin: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., layer: _Optional[str] = ...) -> None: ...

class Costmap(_message.Message):
    __slots__ = ("header", "metadata", "data")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    metadata: CostmapMetaData
    data: bytes
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., metadata: _Optional[_Union[CostmapMetaData, _Mapping]] = ..., data: _Optional[bytes] = ...) -> None: ...

class CostmapFilterInfo(_message.Message):
    __slots__ = ("header", "type", "filter_mask_topic", "base", "multiplier")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    FILTER_MASK_TOPIC_FIELD_NUMBER: _ClassVar[int]
    BASE_FIELD_NUMBER: _ClassVar[int]
    MULTIPLIER_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    type: int
    filter_mask_topic: str
    base: float
    multiplier: float
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., type: _Optional[int] = ..., filter_mask_topic: _Optional[str] = ..., base: _Optional[float] = ..., multiplier: _Optional[float] = ...) -> None: ...
