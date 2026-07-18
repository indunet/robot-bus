from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class JointState(_message.Message):
    __slots__ = ("name", "position", "velocity", "acceleration", "effort")
    NAME_FIELD_NUMBER: _ClassVar[int]
    POSITION_FIELD_NUMBER: _ClassVar[int]
    VELOCITY_FIELD_NUMBER: _ClassVar[int]
    ACCELERATION_FIELD_NUMBER: _ClassVar[int]
    EFFORT_FIELD_NUMBER: _ClassVar[int]
    name: str
    position: float
    velocity: float
    acceleration: float
    effort: float
    def __init__(self, name: _Optional[str] = ..., position: _Optional[float] = ..., velocity: _Optional[float] = ..., acceleration: _Optional[float] = ..., effort: _Optional[float] = ...) -> None: ...
