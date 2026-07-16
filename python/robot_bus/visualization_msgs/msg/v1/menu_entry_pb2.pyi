from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class MenuEntry(_message.Message):
    __slots__ = ("id", "parent_id", "title", "command", "command_type")
    ID_FIELD_NUMBER: _ClassVar[int]
    PARENT_ID_FIELD_NUMBER: _ClassVar[int]
    TITLE_FIELD_NUMBER: _ClassVar[int]
    COMMAND_FIELD_NUMBER: _ClassVar[int]
    COMMAND_TYPE_FIELD_NUMBER: _ClassVar[int]
    id: int
    parent_id: int
    title: str
    command: str
    command_type: int
    def __init__(self, id: _Optional[int] = ..., parent_id: _Optional[int] = ..., title: _Optional[str] = ..., command: _Optional[str] = ..., command_type: _Optional[int] = ...) -> None: ...
