from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PackedElementField(_message.Message):
    __slots__ = ("name", "offset", "type")
    class NumericType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
        __slots__ = ()
        UNKNOWN: _ClassVar[PackedElementField.NumericType]
        UINT8: _ClassVar[PackedElementField.NumericType]
        INT8: _ClassVar[PackedElementField.NumericType]
        UINT16: _ClassVar[PackedElementField.NumericType]
        INT16: _ClassVar[PackedElementField.NumericType]
        UINT32: _ClassVar[PackedElementField.NumericType]
        INT32: _ClassVar[PackedElementField.NumericType]
        FLOAT32: _ClassVar[PackedElementField.NumericType]
        FLOAT64: _ClassVar[PackedElementField.NumericType]
    UNKNOWN: PackedElementField.NumericType
    UINT8: PackedElementField.NumericType
    INT8: PackedElementField.NumericType
    UINT16: PackedElementField.NumericType
    INT16: PackedElementField.NumericType
    UINT32: PackedElementField.NumericType
    INT32: PackedElementField.NumericType
    FLOAT32: PackedElementField.NumericType
    FLOAT64: PackedElementField.NumericType
    NAME_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    name: str
    offset: int
    type: PackedElementField.NumericType
    def __init__(self, name: _Optional[str] = ..., offset: _Optional[int] = ..., type: _Optional[_Union[PackedElementField.NumericType, str]] = ...) -> None: ...
