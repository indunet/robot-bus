from robot_bus.foxglove_msgs.msg.v1 import color_pb2 as _color_pb2
from robot_bus.foxglove_msgs.msg.v1 import pose_pb2 as _pose_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class TextPrimitive(_message.Message):
    __slots__ = ("pose", "billboard", "font_size", "scale_invariant", "color", "text")
    POSE_FIELD_NUMBER: _ClassVar[int]
    BILLBOARD_FIELD_NUMBER: _ClassVar[int]
    FONT_SIZE_FIELD_NUMBER: _ClassVar[int]
    SCALE_INVARIANT_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    pose: _pose_pb2.Pose
    billboard: bool
    font_size: float
    scale_invariant: bool
    color: _color_pb2.Color
    text: str
    def __init__(self, pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., billboard: _Optional[bool] = ..., font_size: _Optional[float] = ..., scale_invariant: _Optional[bool] = ..., color: _Optional[_Union[_color_pb2.Color, _Mapping]] = ..., text: _Optional[str] = ...) -> None: ...
