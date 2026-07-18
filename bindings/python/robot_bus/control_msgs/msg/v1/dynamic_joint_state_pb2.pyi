from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class InterfaceValue(_message.Message):
    __slots__ = ("interface_names", "values")
    INTERFACE_NAMES_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    interface_names: _containers.RepeatedScalarFieldContainer[str]
    values: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, interface_names: _Optional[_Iterable[str]] = ..., values: _Optional[_Iterable[float]] = ...) -> None: ...

class DynamicJointState(_message.Message):
    __slots__ = ("header", "joint_names", "interface_values")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    INTERFACE_VALUES_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    joint_names: _containers.RepeatedScalarFieldContainer[str]
    interface_values: _containers.RepeatedCompositeFieldContainer[InterfaceValue]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., joint_names: _Optional[_Iterable[str]] = ..., interface_values: _Optional[_Iterable[_Union[InterfaceValue, _Mapping]]] = ...) -> None: ...

class DynamicInterfaceGroupValues(_message.Message):
    __slots__ = ("header", "interface_groups", "interface_values")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    INTERFACE_GROUPS_FIELD_NUMBER: _ClassVar[int]
    INTERFACE_VALUES_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    interface_groups: _containers.RepeatedScalarFieldContainer[str]
    interface_values: _containers.RepeatedCompositeFieldContainer[InterfaceValue]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., interface_groups: _Optional[_Iterable[str]] = ..., interface_values: _Optional[_Iterable[_Union[InterfaceValue, _Mapping]]] = ...) -> None: ...
