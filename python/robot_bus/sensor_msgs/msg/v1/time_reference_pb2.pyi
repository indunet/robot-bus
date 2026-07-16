from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.builtin_interfaces.msg.v1 import time_pb2 as _time_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class TimeReference(_message.Message):
    __slots__ = ("header", "time_ref", "source")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    TIME_REF_FIELD_NUMBER: _ClassVar[int]
    SOURCE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    time_ref: _time_pb2.Time
    source: str
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., time_ref: _Optional[_Union[_time_pb2.Time, _Mapping]] = ..., source: _Optional[str] = ...) -> None: ...
