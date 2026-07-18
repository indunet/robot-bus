from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class JointComponentTolerance(_message.Message):
    __slots__ = ("joint_name", "component", "value")
    JOINT_NAME_FIELD_NUMBER: _ClassVar[int]
    COMPONENT_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    joint_name: str
    component: int
    value: float
    def __init__(self, joint_name: _Optional[str] = ..., component: _Optional[int] = ..., value: _Optional[float] = ...) -> None: ...
