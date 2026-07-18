from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Range(_message.Message):
    __slots__ = ("header", "radiation_type", "field_of_view", "min_range", "max_range", "range")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    RADIATION_TYPE_FIELD_NUMBER: _ClassVar[int]
    FIELD_OF_VIEW_FIELD_NUMBER: _ClassVar[int]
    MIN_RANGE_FIELD_NUMBER: _ClassVar[int]
    MAX_RANGE_FIELD_NUMBER: _ClassVar[int]
    RANGE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    radiation_type: int
    field_of_view: float
    min_range: float
    max_range: float
    range: float
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., radiation_type: _Optional[int] = ..., field_of_view: _Optional[float] = ..., min_range: _Optional[float] = ..., max_range: _Optional[float] = ..., range: _Optional[float] = ...) -> None: ...
