from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.std_msgs.msg.v1 import color_rgba_pb2 as _color_rgba_pb2
from robot_bus.builtin_interfaces.msg.v1 import duration_pb2 as _duration_pb2
from robot_bus.geometry_msgs.msg.v1 import pose_pb2 as _pose_pb2
from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from robot_bus.geometry_msgs.msg.v1 import point_pb2 as _point_pb2
from robot_bus.sensor_msgs.msg.v1 import compressed_image_pb2 as _compressed_image_pb2
from robot_bus.visualization_msgs.msg.v1 import uv_coordinate_pb2 as _uv_coordinate_pb2
from robot_bus.visualization_msgs.msg.v1 import mesh_file_pb2 as _mesh_file_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Marker(_message.Message):
    __slots__ = ("header", "ns", "id", "type", "action", "pose", "scale", "color", "lifetime", "frame_locked", "points", "colors", "texture_resource", "texture", "uv_coordinates", "text", "mesh_resource", "mesh_file", "mesh_use_embedded_materials")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    NS_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    ACTION_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    SCALE_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    LIFETIME_FIELD_NUMBER: _ClassVar[int]
    FRAME_LOCKED_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    COLORS_FIELD_NUMBER: _ClassVar[int]
    TEXTURE_RESOURCE_FIELD_NUMBER: _ClassVar[int]
    TEXTURE_FIELD_NUMBER: _ClassVar[int]
    UV_COORDINATES_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    MESH_RESOURCE_FIELD_NUMBER: _ClassVar[int]
    MESH_FILE_FIELD_NUMBER: _ClassVar[int]
    MESH_USE_EMBEDDED_MATERIALS_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    ns: str
    id: int
    type: int
    action: int
    pose: _pose_pb2.Pose
    scale: _vector3_pb2.Vector3
    color: _color_rgba_pb2.ColorRGBA
    lifetime: _duration_pb2.Duration
    frame_locked: bool
    points: _containers.RepeatedCompositeFieldContainer[_point_pb2.Point]
    colors: _containers.RepeatedCompositeFieldContainer[_color_rgba_pb2.ColorRGBA]
    texture_resource: str
    texture: _compressed_image_pb2.CompressedImage
    uv_coordinates: _containers.RepeatedCompositeFieldContainer[_uv_coordinate_pb2.UVCoordinate]
    text: str
    mesh_resource: str
    mesh_file: _mesh_file_pb2.MeshFile
    mesh_use_embedded_materials: bool
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., ns: _Optional[str] = ..., id: _Optional[int] = ..., type: _Optional[int] = ..., action: _Optional[int] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., scale: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., color: _Optional[_Union[_color_rgba_pb2.ColorRGBA, _Mapping]] = ..., lifetime: _Optional[_Union[_duration_pb2.Duration, _Mapping]] = ..., frame_locked: bool = ..., points: _Optional[_Iterable[_Union[_point_pb2.Point, _Mapping]]] = ..., colors: _Optional[_Iterable[_Union[_color_rgba_pb2.ColorRGBA, _Mapping]]] = ..., texture_resource: _Optional[str] = ..., texture: _Optional[_Union[_compressed_image_pb2.CompressedImage, _Mapping]] = ..., uv_coordinates: _Optional[_Iterable[_Union[_uv_coordinate_pb2.UVCoordinate, _Mapping]]] = ..., text: _Optional[str] = ..., mesh_resource: _Optional[str] = ..., mesh_file: _Optional[_Union[_mesh_file_pb2.MeshFile, _Mapping]] = ..., mesh_use_embedded_materials: bool = ...) -> None: ...

class MarkerArray(_message.Message):
    __slots__ = ("markers",)
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    markers: _containers.RepeatedCompositeFieldContainer[Marker]
    def __init__(self, markers: _Optional[_Iterable[_Union[Marker, _Mapping]]] = ...) -> None: ...
