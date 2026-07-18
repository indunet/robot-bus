from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Duration(_message.Message):
    __slots__ = ("sec", "nanosec")
    SEC_FIELD_NUMBER: _ClassVar[int]
    NANOSEC_FIELD_NUMBER: _ClassVar[int]
    sec: int
    nanosec: int
    def __init__(self, sec: _Optional[int] = ..., nanosec: _Optional[int] = ...) -> None: ...
