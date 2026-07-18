from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Illuminance(_message.Message):
    __slots__ = ("header", "illuminance", "variance")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    ILLUMINANCE_FIELD_NUMBER: _ClassVar[int]
    VARIANCE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    illuminance: float
    variance: float
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., illuminance: _Optional[float] = ..., variance: _Optional[float] = ...) -> None: ...
