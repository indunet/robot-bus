from robot_bus.foxglove_msgs.msg.v1 import color_pb2 as _color_pb2
from robot_bus.foxglove_msgs.msg.v1 import point3_pb2 as _point3_pb2
from robot_bus.foxglove_msgs.msg.v1 import pose_pb2 as _pose_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class LinePrimitive(_message.Message):
    __slots__ = ("type", "pose", "thickness", "scale_invariant", "points", "color", "colors", "indices")
    class Type(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
        __slots__ = ()
        LINE_STRIP: _ClassVar[LinePrimitive.Type]
        LINE_LOOP: _ClassVar[LinePrimitive.Type]
        LINE_LIST: _ClassVar[LinePrimitive.Type]
    LINE_STRIP: LinePrimitive.Type
    LINE_LOOP: LinePrimitive.Type
    LINE_LIST: LinePrimitive.Type
    TYPE_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    THICKNESS_FIELD_NUMBER: _ClassVar[int]
    SCALE_INVARIANT_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    COLORS_FIELD_NUMBER: _ClassVar[int]
    INDICES_FIELD_NUMBER: _ClassVar[int]
    type: LinePrimitive.Type
    pose: _pose_pb2.Pose
    thickness: float
    scale_invariant: bool
    points: _containers.RepeatedCompositeFieldContainer[_point3_pb2.Point3]
    color: _color_pb2.Color
    colors: _containers.RepeatedCompositeFieldContainer[_color_pb2.Color]
    indices: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, type: _Optional[_Union[LinePrimitive.Type, str]] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., thickness: _Optional[float] = ..., scale_invariant: bool = ..., points: _Optional[_Iterable[_Union[_point3_pb2.Point3, _Mapping]]] = ..., color: _Optional[_Union[_color_pb2.Color, _Mapping]] = ..., colors: _Optional[_Iterable[_Union[_color_pb2.Color, _Mapping]]] = ..., indices: _Optional[_Iterable[int]] = ...) -> None: ...
