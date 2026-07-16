from robot_bus.builtin_interfaces.msg.v1 import time_pb2 as _time_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Header(_message.Message):
    __slots__ = ("stamp", "frame_id")
    STAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    stamp: _time_pb2.Time
    frame_id: str
    def __init__(self, stamp: _Optional[_Union[_time_pb2.Time, _Mapping]] = ..., frame_id: _Optional[str] = ...) -> None: ...
