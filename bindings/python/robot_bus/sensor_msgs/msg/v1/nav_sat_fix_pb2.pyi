from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.sensor_msgs.msg.v1 import nav_sat_status_pb2 as _nav_sat_status_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class NavSatFix(_message.Message):
    __slots__ = ("header", "status", "latitude", "longitude", "altitude", "position_covariance", "position_covariance_type")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    LATITUDE_FIELD_NUMBER: _ClassVar[int]
    LONGITUDE_FIELD_NUMBER: _ClassVar[int]
    ALTITUDE_FIELD_NUMBER: _ClassVar[int]
    POSITION_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    POSITION_COVARIANCE_TYPE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    status: _nav_sat_status_pb2.NavSatStatus
    latitude: float
    longitude: float
    altitude: float
    position_covariance: _containers.RepeatedScalarFieldContainer[float]
    position_covariance_type: int
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., status: _Optional[_Union[_nav_sat_status_pb2.NavSatStatus, _Mapping]] = ..., latitude: _Optional[float] = ..., longitude: _Optional[float] = ..., altitude: _Optional[float] = ..., position_covariance: _Optional[_Iterable[float]] = ..., position_covariance_type: _Optional[int] = ...) -> None: ...
