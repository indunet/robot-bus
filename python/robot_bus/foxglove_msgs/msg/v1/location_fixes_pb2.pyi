from robot_bus.foxglove_msgs.msg.v1 import location_fix_pb2 as _location_fix_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class LocationFixes(_message.Message):
    __slots__ = ("fixes",)
    FIXES_FIELD_NUMBER: _ClassVar[int]
    fixes: _containers.RepeatedCompositeFieldContainer[_location_fix_pb2.LocationFix]
    def __init__(self, fixes: _Optional[_Iterable[_Union[_location_fix_pb2.LocationFix, _Mapping]]] = ...) -> None: ...
