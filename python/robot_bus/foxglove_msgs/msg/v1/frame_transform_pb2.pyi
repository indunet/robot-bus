import datetime

from robot_bus.foxglove_msgs.msg.v1 import quaternion_pb2 as _quaternion_pb2
from robot_bus.foxglove_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class FrameTransform(_message.Message):
    __slots__ = ("timestamp", "parent_frame_id", "child_frame_id", "translation", "rotation")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    PARENT_FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    CHILD_FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    TRANSLATION_FIELD_NUMBER: _ClassVar[int]
    ROTATION_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    parent_frame_id: str
    child_frame_id: str
    translation: _vector3_pb2.Vector3
    rotation: _quaternion_pb2.Quaternion
    def __init__(self, timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., parent_frame_id: _Optional[str] = ..., child_frame_id: _Optional[str] = ..., translation: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., rotation: _Optional[_Union[_quaternion_pb2.Quaternion, _Mapping]] = ...) -> None: ...
