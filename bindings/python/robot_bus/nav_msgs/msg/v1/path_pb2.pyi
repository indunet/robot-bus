from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import stamped_pb2 as _stamped_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Path(_message.Message):
    __slots__ = ("header", "poses")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POSES_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    poses: _containers.RepeatedCompositeFieldContainer[_stamped_pb2.PoseStamped]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., poses: _Optional[_Iterable[_Union[_stamped_pb2.PoseStamped, _Mapping]]] = ...) -> None: ...
