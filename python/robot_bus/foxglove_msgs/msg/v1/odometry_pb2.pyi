from robot_bus.foxglove_msgs.msg.v1 import key_value_pair_pb2 as _key_value_pair_pb2
from robot_bus.foxglove_msgs.msg.v1 import pose_pb2 as _pose_pb2
from robot_bus.foxglove_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Odometry(_message.Message):
    __slots__ = ("timestamp", "frame_id", "body_frame_id", "pose", "linear_velocity", "angular_velocity", "pose_covariance", "velocity_covariance", "metadata")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    BODY_FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    LINEAR_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    ANGULAR_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    POSE_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    VELOCITY_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: str
    body_frame_id: str
    pose: _pose_pb2.Pose
    linear_velocity: _vector3_pb2.Vector3
    angular_velocity: _vector3_pb2.Vector3
    pose_covariance: _containers.RepeatedScalarFieldContainer[float]
    velocity_covariance: _containers.RepeatedScalarFieldContainer[float]
    metadata: _containers.RepeatedCompositeFieldContainer[_key_value_pair_pb2.KeyValuePair]
    def __init__(self, timestamp: _Optional[_Union[_timestamp_pb2.Timestamp, _Mapping]] = ..., frame_id: _Optional[str] = ..., body_frame_id: _Optional[str] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., linear_velocity: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., angular_velocity: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., pose_covariance: _Optional[_Iterable[float]] = ..., velocity_covariance: _Optional[_Iterable[float]] = ..., metadata: _Optional[_Iterable[_Union[_key_value_pair_pb2.KeyValuePair, _Mapping]]] = ...) -> None: ...
