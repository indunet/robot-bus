from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class CollisionMonitorState(_message.Message):
    __slots__ = ("action_type", "polygon_name")
    ACTION_TYPE_FIELD_NUMBER: _ClassVar[int]
    POLYGON_NAME_FIELD_NUMBER: _ClassVar[int]
    action_type: int
    polygon_name: str
    def __init__(self, action_type: _Optional[int] = ..., polygon_name: _Optional[str] = ...) -> None: ...
