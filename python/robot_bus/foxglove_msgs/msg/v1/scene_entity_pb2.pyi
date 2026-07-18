from robot_bus.foxglove_msgs.msg.v1 import arrow_primitive_pb2 as _arrow_primitive_pb2
from robot_bus.foxglove_msgs.msg.v1 import cube_primitive_pb2 as _cube_primitive_pb2
from robot_bus.foxglove_msgs.msg.v1 import cylinder_primitive_pb2 as _cylinder_primitive_pb2
from robot_bus.foxglove_msgs.msg.v1 import key_value_pair_pb2 as _key_value_pair_pb2
from robot_bus.foxglove_msgs.msg.v1 import line_primitive_pb2 as _line_primitive_pb2
from robot_bus.foxglove_msgs.msg.v1 import model_primitive_pb2 as _model_primitive_pb2
from robot_bus.foxglove_msgs.msg.v1 import sphere_primitive_pb2 as _sphere_primitive_pb2
from robot_bus.foxglove_msgs.msg.v1 import text_primitive_pb2 as _text_primitive_pb2
from robot_bus.foxglove_msgs.msg.v1 import triangle_list_primitive_pb2 as _triangle_list_primitive_pb2
from google.protobuf import duration_pb2 as _duration_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SceneEntity(_message.Message):
    __slots__ = ("timestamp", "frame_id", "id", "lifetime", "frame_locked", "metadata", "arrows", "cubes", "spheres", "cylinders", "lines", "triangles", "texts", "models")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    LIFETIME_FIELD_NUMBER: _ClassVar[int]
    FRAME_LOCKED_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ARROWS_FIELD_NUMBER: _ClassVar[int]
    CUBES_FIELD_NUMBER: _ClassVar[int]
    SPHERES_FIELD_NUMBER: _ClassVar[int]
    CYLINDERS_FIELD_NUMBER: _ClassVar[int]
    LINES_FIELD_NUMBER: _ClassVar[int]
    TRIANGLES_FIELD_NUMBER: _ClassVar[int]
    TEXTS_FIELD_NUMBER: _ClassVar[int]
    MODELS_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: str
    id: str
    lifetime: _duration_pb2.Duration
    frame_locked: bool
    metadata: _containers.RepeatedCompositeFieldContainer[_key_value_pair_pb2.KeyValuePair]
    arrows: _containers.RepeatedCompositeFieldContainer[_arrow_primitive_pb2.ArrowPrimitive]
    cubes: _containers.RepeatedCompositeFieldContainer[_cube_primitive_pb2.CubePrimitive]
    spheres: _containers.RepeatedCompositeFieldContainer[_sphere_primitive_pb2.SpherePrimitive]
    cylinders: _containers.RepeatedCompositeFieldContainer[_cylinder_primitive_pb2.CylinderPrimitive]
    lines: _containers.RepeatedCompositeFieldContainer[_line_primitive_pb2.LinePrimitive]
    triangles: _containers.RepeatedCompositeFieldContainer[_triangle_list_primitive_pb2.TriangleListPrimitive]
    texts: _containers.RepeatedCompositeFieldContainer[_text_primitive_pb2.TextPrimitive]
    models: _containers.RepeatedCompositeFieldContainer[_model_primitive_pb2.ModelPrimitive]
    def __init__(self, timestamp: _Optional[_Union[_timestamp_pb2.Timestamp, _Mapping]] = ..., frame_id: _Optional[str] = ..., id: _Optional[str] = ..., lifetime: _Optional[_Union[_duration_pb2.Duration, _Mapping]] = ..., frame_locked: bool = ..., metadata: _Optional[_Iterable[_Union[_key_value_pair_pb2.KeyValuePair, _Mapping]]] = ..., arrows: _Optional[_Iterable[_Union[_arrow_primitive_pb2.ArrowPrimitive, _Mapping]]] = ..., cubes: _Optional[_Iterable[_Union[_cube_primitive_pb2.CubePrimitive, _Mapping]]] = ..., spheres: _Optional[_Iterable[_Union[_sphere_primitive_pb2.SpherePrimitive, _Mapping]]] = ..., cylinders: _Optional[_Iterable[_Union[_cylinder_primitive_pb2.CylinderPrimitive, _Mapping]]] = ..., lines: _Optional[_Iterable[_Union[_line_primitive_pb2.LinePrimitive, _Mapping]]] = ..., triangles: _Optional[_Iterable[_Union[_triangle_list_primitive_pb2.TriangleListPrimitive, _Mapping]]] = ..., texts: _Optional[_Iterable[_Union[_text_primitive_pb2.TextPrimitive, _Mapping]]] = ..., models: _Optional[_Iterable[_Union[_model_primitive_pb2.ModelPrimitive, _Mapping]]] = ...) -> None: ...
