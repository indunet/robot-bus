from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import accel_wrench_pb2 as _accel_wrench_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class AccelWithCovariance(_message.Message):
    __slots__ = ("accel", "covariance")
    ACCEL_FIELD_NUMBER: _ClassVar[int]
    COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    accel: _accel_wrench_pb2.Accel
    covariance: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, accel: _Optional[_Union[_accel_wrench_pb2.Accel, _Mapping]] = ..., covariance: _Optional[_Iterable[float]] = ...) -> None: ...

class AccelWithCovarianceStamped(_message.Message):
    __slots__ = ("header", "accel")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    ACCEL_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    accel: AccelWithCovariance
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., accel: _Optional[_Union[AccelWithCovariance, _Mapping]] = ...) -> None: ...
