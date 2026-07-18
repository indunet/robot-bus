from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class GripperCommand(_message.Message):
    __slots__ = ("position", "max_effort")
    POSITION_FIELD_NUMBER: _ClassVar[int]
    MAX_EFFORT_FIELD_NUMBER: _ClassVar[int]
    position: float
    max_effort: float
    def __init__(self, position: _Optional[float] = ..., max_effort: _Optional[float] = ...) -> None: ...
