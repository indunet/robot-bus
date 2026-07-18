from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.diagnostic_msgs.msg.v1 import key_value_pb2 as _key_value_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class DiagnosticStatus(_message.Message):
    __slots__ = ("level", "name", "message", "hardware_id", "values")
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    HARDWARE_ID_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    level: int
    name: str
    message: str
    hardware_id: str
    values: _containers.RepeatedCompositeFieldContainer[_key_value_pb2.KeyValue]
    def __init__(self, level: _Optional[int] = ..., name: _Optional[str] = ..., message: _Optional[str] = ..., hardware_id: _Optional[str] = ..., values: _Optional[_Iterable[_Union[_key_value_pb2.KeyValue, _Mapping]]] = ...) -> None: ...

class DiagnosticArray(_message.Message):
    __slots__ = ("header", "status")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    status: _containers.RepeatedCompositeFieldContainer[DiagnosticStatus]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., status: _Optional[_Iterable[_Union[DiagnosticStatus, _Mapping]]] = ...) -> None: ...
