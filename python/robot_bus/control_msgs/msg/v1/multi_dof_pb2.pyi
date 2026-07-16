from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.control_msgs.msg.v1 import single_dof_state_pb2 as _single_dof_state_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MultiDOFCommand(_message.Message):
    __slots__ = ("dof_names", "values", "values_dot")
    DOF_NAMES_FIELD_NUMBER: _ClassVar[int]
    VALUES_FIELD_NUMBER: _ClassVar[int]
    VALUES_DOT_FIELD_NUMBER: _ClassVar[int]
    dof_names: _containers.RepeatedScalarFieldContainer[str]
    values: _containers.RepeatedScalarFieldContainer[float]
    values_dot: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, dof_names: _Optional[_Iterable[str]] = ..., values: _Optional[_Iterable[float]] = ..., values_dot: _Optional[_Iterable[float]] = ...) -> None: ...

class MultiDOFStateStamped(_message.Message):
    __slots__ = ("header", "dof_states")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    DOF_STATES_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    dof_states: _containers.RepeatedCompositeFieldContainer[_single_dof_state_pb2.SingleDOFState]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., dof_states: _Optional[_Iterable[_Union[_single_dof_state_pb2.SingleDOFState, _Mapping]]] = ...) -> None: ...
