import datetime

from robot_bus.foxglove_msgs.msg.v1 import pose_pb2 as _pose_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class LaserScan(_message.Message):
    __slots__ = ("timestamp", "frame_id", "pose", "start_angle", "end_angle", "ranges", "intensities")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    START_ANGLE_FIELD_NUMBER: _ClassVar[int]
    END_ANGLE_FIELD_NUMBER: _ClassVar[int]
    RANGES_FIELD_NUMBER: _ClassVar[int]
    INTENSITIES_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: str
    pose: _pose_pb2.Pose
    start_angle: float
    end_angle: float
    ranges: _containers.RepeatedScalarFieldContainer[float]
    intensities: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., frame_id: _Optional[str] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., start_angle: _Optional[float] = ..., end_angle: _Optional[float] = ..., ranges: _Optional[_Iterable[float]] = ..., intensities: _Optional[_Iterable[float]] = ...) -> None: ...
