from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class FibonacciGoal(_message.Message):
    __slots__ = ("order",)
    ORDER_FIELD_NUMBER: _ClassVar[int]
    order: int
    def __init__(self, order: _Optional[int] = ...) -> None: ...

class FibonacciFeedback(_message.Message):
    __slots__ = ("sequence",)
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    sequence: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, sequence: _Optional[_Iterable[int]] = ...) -> None: ...

class FibonacciResult(_message.Message):
    __slots__ = ("sequence",)
    SEQUENCE_FIELD_NUMBER: _ClassVar[int]
    sequence: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, sequence: _Optional[_Iterable[int]] = ...) -> None: ...
