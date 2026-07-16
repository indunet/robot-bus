from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RegionOfInterest(_message.Message):
    __slots__ = ("x_offset", "y_offset", "height", "width", "do_rectify")
    X_OFFSET_FIELD_NUMBER: _ClassVar[int]
    Y_OFFSET_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    DO_RECTIFY_FIELD_NUMBER: _ClassVar[int]
    x_offset: int
    y_offset: int
    height: int
    width: int
    do_rectify: bool
    def __init__(self, x_offset: _Optional[int] = ..., y_offset: _Optional[int] = ..., height: _Optional[int] = ..., width: _Optional[int] = ..., do_rectify: bool = ...) -> None: ...

class CameraInfo(_message.Message):
    __slots__ = ("header", "height", "width", "distortion_model", "d", "k", "r", "p", "binning_x", "binning_y", "roi")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    DISTORTION_MODEL_FIELD_NUMBER: _ClassVar[int]
    D_FIELD_NUMBER: _ClassVar[int]
    K_FIELD_NUMBER: _ClassVar[int]
    R_FIELD_NUMBER: _ClassVar[int]
    P_FIELD_NUMBER: _ClassVar[int]
    BINNING_X_FIELD_NUMBER: _ClassVar[int]
    BINNING_Y_FIELD_NUMBER: _ClassVar[int]
    ROI_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    height: int
    width: int
    distortion_model: str
    d: _containers.RepeatedScalarFieldContainer[float]
    k: _containers.RepeatedScalarFieldContainer[float]
    r: _containers.RepeatedScalarFieldContainer[float]
    p: _containers.RepeatedScalarFieldContainer[float]
    binning_x: int
    binning_y: int
    roi: RegionOfInterest
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., height: _Optional[int] = ..., width: _Optional[int] = ..., distortion_model: _Optional[str] = ..., d: _Optional[_Iterable[float]] = ..., k: _Optional[_Iterable[float]] = ..., r: _Optional[_Iterable[float]] = ..., p: _Optional[_Iterable[float]] = ..., binning_x: _Optional[int] = ..., binning_y: _Optional[int] = ..., roi: _Optional[_Union[RegionOfInterest, _Mapping]] = ...) -> None: ...
