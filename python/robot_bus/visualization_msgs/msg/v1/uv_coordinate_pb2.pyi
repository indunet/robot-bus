from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class UVCoordinate(_message.Message):
    __slots__ = ("u", "v")
    U_FIELD_NUMBER: _ClassVar[int]
    V_FIELD_NUMBER: _ClassVar[int]
    u: float
    v: float
    def __init__(self, u: _Optional[float] = ..., v: _Optional[float] = ...) -> None: ...
