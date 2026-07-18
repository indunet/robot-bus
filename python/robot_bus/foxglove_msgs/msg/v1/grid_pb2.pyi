from robot_bus.foxglove_msgs.msg.v1 import packed_element_field_pb2 as _packed_element_field_pb2
from robot_bus.foxglove_msgs.msg.v1 import pose_pb2 as _pose_pb2
from robot_bus.foxglove_msgs.msg.v1 import vector2_pb2 as _vector2_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Grid(_message.Message):
    __slots__ = ("timestamp", "frame_id", "pose", "column_count", "cell_size", "row_stride", "cell_stride", "fields", "data")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    COLUMN_COUNT_FIELD_NUMBER: _ClassVar[int]
    CELL_SIZE_FIELD_NUMBER: _ClassVar[int]
    ROW_STRIDE_FIELD_NUMBER: _ClassVar[int]
    CELL_STRIDE_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: str
    pose: _pose_pb2.Pose
    column_count: int
    cell_size: _vector2_pb2.Vector2
    row_stride: int
    cell_stride: int
    fields: _containers.RepeatedCompositeFieldContainer[_packed_element_field_pb2.PackedElementField]
    data: bytes
    def __init__(self, timestamp: _Optional[_Union[_timestamp_pb2.Timestamp, _Mapping]] = ..., frame_id: _Optional[str] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., column_count: _Optional[int] = ..., cell_size: _Optional[_Union[_vector2_pb2.Vector2, _Mapping]] = ..., row_stride: _Optional[int] = ..., cell_stride: _Optional[int] = ..., fields: _Optional[_Iterable[_Union[_packed_element_field_pb2.PackedElementField, _Mapping]]] = ..., data: _Optional[bytes] = ...) -> None: ...
