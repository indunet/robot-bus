from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MultiArrayDimension(_message.Message):
    __slots__ = ("label", "size", "stride")
    LABEL_FIELD_NUMBER: _ClassVar[int]
    SIZE_FIELD_NUMBER: _ClassVar[int]
    STRIDE_FIELD_NUMBER: _ClassVar[int]
    label: str
    size: int
    stride: int
    def __init__(self, label: _Optional[str] = ..., size: _Optional[int] = ..., stride: _Optional[int] = ...) -> None: ...

class MultiArrayLayout(_message.Message):
    __slots__ = ("dim", "data_offset")
    DIM_FIELD_NUMBER: _ClassVar[int]
    DATA_OFFSET_FIELD_NUMBER: _ClassVar[int]
    dim: _containers.RepeatedCompositeFieldContainer[MultiArrayDimension]
    data_offset: int
    def __init__(self, dim: _Optional[_Iterable[_Union[MultiArrayDimension, _Mapping]]] = ..., data_offset: _Optional[int] = ...) -> None: ...
