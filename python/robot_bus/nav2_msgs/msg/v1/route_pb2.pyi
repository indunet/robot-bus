from robot_bus.geometry_msgs.msg.v1 import point_pb2 as _point_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RouteNode(_message.Message):
    __slots__ = ("nodeid", "position")
    NODEID_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    nodeid: int
    position: _point_pb2.Point
    def __init__(self, nodeid: _Optional[int] = ..., position: _Optional[_Union[_point_pb2.Point, _Mapping]] = ...) -> None: ...

class RouteEdge(_message.Message):
    __slots__ = ("edgeid", "start", "end")
    EDGEID_FIELD_NUMBER: _ClassVar[int]
    START_FIELD_NUMBER: _ClassVar[int]
    END_FIELD_NUMBER: _ClassVar[int]
    edgeid: int
    start: int
    end: int
    def __init__(self, edgeid: _Optional[int] = ..., start: _Optional[int] = ..., end: _Optional[int] = ...) -> None: ...

class EdgeCost(_message.Message):
    __slots__ = ("edgeid", "cost")
    EDGEID_FIELD_NUMBER: _ClassVar[int]
    COST_FIELD_NUMBER: _ClassVar[int]
    edgeid: int
    cost: float
    def __init__(self, edgeid: _Optional[int] = ..., cost: _Optional[float] = ...) -> None: ...

class Route(_message.Message):
    __slots__ = ("nodes", "edges")
    NODES_FIELD_NUMBER: _ClassVar[int]
    EDGES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[RouteNode]
    edges: _containers.RepeatedCompositeFieldContainer[RouteEdge]
    def __init__(self, nodes: _Optional[_Iterable[_Union[RouteNode, _Mapping]]] = ..., edges: _Optional[_Iterable[_Union[RouteEdge, _Mapping]]] = ...) -> None: ...
