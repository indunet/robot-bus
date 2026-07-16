from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PointField(_message.Message):
    __slots__ = ("name", "offset", "datatype", "count")
    NAME_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    DATATYPE_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    name: str
    offset: int
    datatype: int
    count: int
    def __init__(self, name: _Optional[str] = ..., offset: _Optional[int] = ..., datatype: _Optional[int] = ..., count: _Optional[int] = ...) -> None: ...

class PointCloud2(_message.Message):
    __slots__ = ("header", "height", "width", "fields", "is_bigendian", "point_step", "row_step", "data", "is_dense")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    FIELDS_FIELD_NUMBER: _ClassVar[int]
    IS_BIGENDIAN_FIELD_NUMBER: _ClassVar[int]
    POINT_STEP_FIELD_NUMBER: _ClassVar[int]
    ROW_STEP_FIELD_NUMBER: _ClassVar[int]
    DATA_FIELD_NUMBER: _ClassVar[int]
    IS_DENSE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    height: int
    width: int
    fields: _containers.RepeatedCompositeFieldContainer[PointField]
    is_bigendian: bool
    point_step: int
    row_step: int
    data: bytes
    is_dense: bool
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., height: _Optional[int] = ..., width: _Optional[int] = ..., fields: _Optional[_Iterable[_Union[PointField, _Mapping]]] = ..., is_bigendian: bool = ..., point_step: _Optional[int] = ..., row_step: _Optional[int] = ..., data: _Optional[bytes] = ..., is_dense: bool = ...) -> None: ...
