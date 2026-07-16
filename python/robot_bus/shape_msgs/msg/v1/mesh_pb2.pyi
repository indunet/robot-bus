from robot_bus.geometry_msgs.msg.v1 import point_pb2 as _point_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MeshTriangle(_message.Message):
    __slots__ = ("vertex_indices",)
    VERTEX_INDICES_FIELD_NUMBER: _ClassVar[int]
    vertex_indices: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, vertex_indices: _Optional[_Iterable[int]] = ...) -> None: ...

class Mesh(_message.Message):
    __slots__ = ("triangles", "vertices")
    TRIANGLES_FIELD_NUMBER: _ClassVar[int]
    VERTICES_FIELD_NUMBER: _ClassVar[int]
    triangles: _containers.RepeatedCompositeFieldContainer[MeshTriangle]
    vertices: _containers.RepeatedCompositeFieldContainer[_point_pb2.Point]
    def __init__(self, triangles: _Optional[_Iterable[_Union[MeshTriangle, _Mapping]]] = ..., vertices: _Optional[_Iterable[_Union[_point_pb2.Point, _Mapping]]] = ...) -> None: ...
