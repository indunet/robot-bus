from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class Plane(_message.Message):
    __slots__ = ("coef",)
    COEF_FIELD_NUMBER: _ClassVar[int]
    coef: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, coef: _Optional[_Iterable[float]] = ...) -> None: ...
