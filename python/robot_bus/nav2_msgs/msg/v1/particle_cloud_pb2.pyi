from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import pose_pb2 as _pose_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Particle(_message.Message):
    __slots__ = ("pose", "weight")
    POSE_FIELD_NUMBER: _ClassVar[int]
    WEIGHT_FIELD_NUMBER: _ClassVar[int]
    pose: _pose_pb2.Pose
    weight: float
    def __init__(self, pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., weight: _Optional[float] = ...) -> None: ...

class ParticleCloud(_message.Message):
    __slots__ = ("header", "particles")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    PARTICLES_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    particles: _containers.RepeatedCompositeFieldContainer[Particle]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., particles: _Optional[_Iterable[_Union[Particle, _Mapping]]] = ...) -> None: ...
