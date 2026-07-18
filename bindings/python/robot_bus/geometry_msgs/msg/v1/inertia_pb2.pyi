from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Inertia(_message.Message):
    __slots__ = ("m", "com", "ixx", "ixy", "ixz", "iyy", "iyz", "izz")
    M_FIELD_NUMBER: _ClassVar[int]
    COM_FIELD_NUMBER: _ClassVar[int]
    IXX_FIELD_NUMBER: _ClassVar[int]
    IXY_FIELD_NUMBER: _ClassVar[int]
    IXZ_FIELD_NUMBER: _ClassVar[int]
    IYY_FIELD_NUMBER: _ClassVar[int]
    IYZ_FIELD_NUMBER: _ClassVar[int]
    IZZ_FIELD_NUMBER: _ClassVar[int]
    m: float
    com: _vector3_pb2.Vector3
    ixx: float
    ixy: float
    ixz: float
    iyy: float
    iyz: float
    izz: float
    def __init__(self, m: _Optional[float] = ..., com: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., ixx: _Optional[float] = ..., ixy: _Optional[float] = ..., ixz: _Optional[float] = ..., iyy: _Optional[float] = ..., iyz: _Optional[float] = ..., izz: _Optional[float] = ...) -> None: ...

class InertiaStamped(_message.Message):
    __slots__ = ("header", "inertia")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    INERTIA_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    inertia: Inertia
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., inertia: _Optional[_Union[Inertia, _Mapping]] = ...) -> None: ...
