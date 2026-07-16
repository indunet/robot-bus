from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class LaserEcho(_message.Message):
    __slots__ = ("echoes",)
    ECHOES_FIELD_NUMBER: _ClassVar[int]
    echoes: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, echoes: _Optional[_Iterable[float]] = ...) -> None: ...

class MultiEchoLaserScan(_message.Message):
    __slots__ = ("header", "angle_min", "angle_max", "angle_increment", "time_increment", "scan_time", "range_min", "range_max", "ranges", "intensities")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    ANGLE_MIN_FIELD_NUMBER: _ClassVar[int]
    ANGLE_MAX_FIELD_NUMBER: _ClassVar[int]
    ANGLE_INCREMENT_FIELD_NUMBER: _ClassVar[int]
    TIME_INCREMENT_FIELD_NUMBER: _ClassVar[int]
    SCAN_TIME_FIELD_NUMBER: _ClassVar[int]
    RANGE_MIN_FIELD_NUMBER: _ClassVar[int]
    RANGE_MAX_FIELD_NUMBER: _ClassVar[int]
    RANGES_FIELD_NUMBER: _ClassVar[int]
    INTENSITIES_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    angle_min: float
    angle_max: float
    angle_increment: float
    time_increment: float
    scan_time: float
    range_min: float
    range_max: float
    ranges: _containers.RepeatedCompositeFieldContainer[LaserEcho]
    intensities: _containers.RepeatedCompositeFieldContainer[LaserEcho]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., angle_min: _Optional[float] = ..., angle_max: _Optional[float] = ..., angle_increment: _Optional[float] = ..., time_increment: _Optional[float] = ..., scan_time: _Optional[float] = ..., range_min: _Optional[float] = ..., range_max: _Optional[float] = ..., ranges: _Optional[_Iterable[_Union[LaserEcho, _Mapping]]] = ..., intensities: _Optional[_Iterable[_Union[LaserEcho, _Mapping]]] = ...) -> None: ...
