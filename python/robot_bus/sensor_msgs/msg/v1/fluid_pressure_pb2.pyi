from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class FluidPressure(_message.Message):
    __slots__ = ("header", "fluid_pressure", "variance")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    FLUID_PRESSURE_FIELD_NUMBER: _ClassVar[int]
    VARIANCE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    fluid_pressure: float
    variance: float
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., fluid_pressure: _Optional[float] = ..., variance: _Optional[float] = ...) -> None: ...
