import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CameraCalibration(_message.Message):
    __slots__ = ("timestamp", "frame_id", "width", "height", "distortion_model", "D", "K", "R", "P")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    WIDTH_FIELD_NUMBER: _ClassVar[int]
    HEIGHT_FIELD_NUMBER: _ClassVar[int]
    DISTORTION_MODEL_FIELD_NUMBER: _ClassVar[int]
    D_FIELD_NUMBER: _ClassVar[int]
    K_FIELD_NUMBER: _ClassVar[int]
    R_FIELD_NUMBER: _ClassVar[int]
    P_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: str
    width: int
    height: int
    distortion_model: str
    D: _containers.RepeatedScalarFieldContainer[float]
    K: _containers.RepeatedScalarFieldContainer[float]
    R: _containers.RepeatedScalarFieldContainer[float]
    P: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., frame_id: _Optional[str] = ..., width: _Optional[int] = ..., height: _Optional[int] = ..., distortion_model: _Optional[str] = ..., D: _Optional[_Iterable[float]] = ..., K: _Optional[_Iterable[float]] = ..., R: _Optional[_Iterable[float]] = ..., P: _Optional[_Iterable[float]] = ...) -> None: ...
