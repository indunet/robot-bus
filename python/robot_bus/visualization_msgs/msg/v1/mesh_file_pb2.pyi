from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class MeshFile(_message.Message):
    __slots__ = ("filename", "data")
    FILENAME_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    filename: str
    data: bytes
    def __init__(self, filename: _Optional[str] = ..., data: _Optional[bytes] = ...) -> None: ...
