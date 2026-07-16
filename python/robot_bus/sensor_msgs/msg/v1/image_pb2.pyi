from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Image(_message.Message):
    __slots__ = ("header", "height", "width", "encoding", "is_bigendian", "step", "data")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    ENCODING_FIELD_NUMBER: _ClassVar[int]
    IS_BIGENDIAN_FIELD_NUMBER: _ClassVar[int]
    STEP_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    height: int
    width: int
    encoding: str
    is_bigendian: bool
    step: int
    data: bytes
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., height: _Optional[int] = ..., width: _Optional[int] = ..., encoding: _Optional[str] = ..., is_bigendian: bool = ..., step: _Optional[int] = ..., data: _Optional[bytes] = ...) -> None: ...
