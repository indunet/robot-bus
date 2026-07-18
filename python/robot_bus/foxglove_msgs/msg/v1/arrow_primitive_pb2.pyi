from robot_bus.foxglove_msgs.msg.v1 import color_pb2 as _color_pb2
from robot_bus.foxglove_msgs.msg.v1 import pose_pb2 as _pose_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ArrowPrimitive(_message.Message):
    __slots__ = ("pose", "shaft_length", "shaft_diameter", "head_length", "head_diameter", "color")
    POSE_FIELD_NUMBER: _ClassVar[int]
    SHAFT_LENGTH_FIELD_NUMBER: _ClassVar[int]
    SHAFT_DIAMETER_FIELD_NUMBER: _ClassVar[int]
    HEAD_LENGTH_FIELD_NUMBER: _ClassVar[int]
    HEAD_DIAMETER_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    pose: _pose_pb2.Pose
    shaft_length: float
    shaft_diameter: float
    head_length: float
    head_diameter: float
    color: _color_pb2.Color
    def __init__(self, pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., shaft_length: _Optional[float] = ..., shaft_diameter: _Optional[float] = ..., head_length: _Optional[float] = ..., head_diameter: _Optional[float] = ..., color: _Optional[_Union[_color_pb2.Color, _Mapping]] = ...) -> None: ...
