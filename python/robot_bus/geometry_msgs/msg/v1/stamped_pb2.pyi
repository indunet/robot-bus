from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import point_pb2 as _point_pb2
from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from robot_bus.geometry_msgs.msg.v1 import quaternion_pb2 as _quaternion_pb2
from robot_bus.geometry_msgs.msg.v1 import pose_pb2 as _pose_pb2
from robot_bus.geometry_msgs.msg.v1 import twist_pb2 as _twist_pb2
from robot_bus.geometry_msgs.msg.v1 import transform_pb2 as _transform_pb2
from robot_bus.geometry_msgs.msg.v1 import accel_wrench_pb2 as _accel_wrench_pb2
from robot_bus.geometry_msgs.msg.v1 import covariance_pb2 as _covariance_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class PointStamped(_message.Message):
    __slots__ = ("header", "point")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POINT_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    point: _point_pb2.Point
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., point: _Optional[_Union[_point_pb2.Point, _Mapping]] = ...) -> None: ...

class Vector3Stamped(_message.Message):
    __slots__ = ("header", "vector")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    vector: _vector3_pb2.Vector3
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., vector: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ...) -> None: ...

class QuaternionStamped(_message.Message):
    __slots__ = ("header", "quaternion")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    QUATERNION_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    quaternion: _quaternion_pb2.Quaternion
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., quaternion: _Optional[_Union[_quaternion_pb2.Quaternion, _Mapping]] = ...) -> None: ...

class PoseStamped(_message.Message):
    __slots__ = ("header", "pose")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    pose: _pose_pb2.Pose
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ...) -> None: ...

class TwistStamped(_message.Message):
    __slots__ = ("header", "twist")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    TWIST_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    twist: _twist_pb2.Twist
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., twist: _Optional[_Union[_twist_pb2.Twist, _Mapping]] = ...) -> None: ...

class TransformStamped(_message.Message):
    __slots__ = ("header", "child_frame_id", "transform")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    CHILD_FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    TRANSFORM_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    child_frame_id: str
    transform: _transform_pb2.Transform
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., child_frame_id: _Optional[str] = ..., transform: _Optional[_Union[_transform_pb2.Transform, _Mapping]] = ...) -> None: ...

class AccelStamped(_message.Message):
    __slots__ = ("header", "accel")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    ACCEL_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    accel: _accel_wrench_pb2.Accel
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., accel: _Optional[_Union[_accel_wrench_pb2.Accel, _Mapping]] = ...) -> None: ...

class WrenchStamped(_message.Message):
    __slots__ = ("header", "wrench")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    WRENCH_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    wrench: _accel_wrench_pb2.Wrench
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., wrench: _Optional[_Union[_accel_wrench_pb2.Wrench, _Mapping]] = ...) -> None: ...

class PoseWithCovarianceStamped(_message.Message):
    __slots__ = ("header", "pose")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    pose: _covariance_pb2.PoseWithCovariance
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., pose: _Optional[_Union[_covariance_pb2.PoseWithCovariance, _Mapping]] = ...) -> None: ...

class TwistWithCovarianceStamped(_message.Message):
    __slots__ = ("header", "twist")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    TWIST_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    twist: _covariance_pb2.TwistWithCovariance
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., twist: _Optional[_Union[_covariance_pb2.TwistWithCovariance, _Mapping]] = ...) -> None: ...
