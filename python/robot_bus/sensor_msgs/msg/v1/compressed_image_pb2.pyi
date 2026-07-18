from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CompressedImage(_message.Message):
    __slots__ = ("header", "format", "data")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    FORMAT_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    format: str
    data: bytes
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., format: _Optional[str] = ..., data: _Optional[bytes] = ...) -> None: ...
