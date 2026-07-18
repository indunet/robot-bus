from robot_bus.foxglove_msgs.msg.v1 import color_pb2 as _color_pb2
from robot_bus.foxglove_msgs.msg.v1 import pose_pb2 as _pose_pb2
from robot_bus.foxglove_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CylinderPrimitive(_message.Message):
    __slots__ = ("pose", "size", "bottom_scale", "top_scale", "color")
    POSE_FIELD_NUMBER: _ClassVar[int]
    SIZE_FIELD_NUMBER: _ClassVar[int]
    BOTTOM_SCALE_FIELD_NUMBER: _ClassVar[int]
    TOP_SCALE_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    pose: _pose_pb2.Pose
    size: _vector3_pb2.Vector3
    bottom_scale: float
    top_scale: float
    color: _color_pb2.Color
    def __init__(self, pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., size: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., bottom_scale: _Optional[float] = ..., top_scale: _Optional[float] = ..., color: _Optional[_Union[_color_pb2.Color, _Mapping]] = ...) -> None: ...
