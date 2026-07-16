from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import covariance_pb2 as _covariance_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Odometry(_message.Message):
    __slots__ = ("header", "child_frame_id", "pose", "twist")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    CHILD_FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    TWIST_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    child_frame_id: str
    pose: _covariance_pb2.PoseWithCovariance
    twist: _covariance_pb2.TwistWithCovariance
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., child_frame_id: _Optional[str] = ..., pose: _Optional[_Union[_covariance_pb2.PoseWithCovariance, _Mapping]] = ..., twist: _Optional[_Union[_covariance_pb2.TwistWithCovariance, _Mapping]] = ...) -> None: ...
