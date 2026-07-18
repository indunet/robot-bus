from robot_bus.std_msgs.msg.v1 import multi_array_msgs_pb2 as _multi_array_msgs_pb2
from robot_bus.geometry_msgs.msg.v1 import stamped_pb2 as _stamped_pb2
from robot_bus.sensor_msgs.msg.v1 import joint_state_pb2 as _joint_state_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class AdmittanceControllerState(_message.Message):
    __slots__ = ("ref_trans_base_fts", "selected_axes", "ft_sensor_frame", "admittance_position", "admittance_acceleration", "admittance_velocity", "wrench_base", "robot_ref_trans_base_fts", "joint_names", "joint_state")
    REF_TRANS_BASE_FTS_FIELD_NUMBER: _ClassVar[int]
    SELECTED_AXES_FIELD_NUMBER: _ClassVar[int]
    FT_SENSOR_FRAME_FIELD_NUMBER: _ClassVar[int]
    ADMITTANCE_POSITION_FIELD_NUMBER: _ClassVar[int]
    ADMITTANCE_ACCELERATION_FIELD_NUMBER: _ClassVar[int]
    ADMITTANCE_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    WRENCH_BASE_FIELD_NUMBER: _ClassVar[int]
    ROBOT_REF_TRANS_BASE_FTS_FIELD_NUMBER: _ClassVar[int]
    JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    JOINT_STATE_FIELD_NUMBER: _ClassVar[int]
    ref_trans_base_fts: _stamped_pb2.TransformStamped
    selected_axes: _multi_array_msgs_pb2.Float64MultiArray
    ft_sensor_frame: _stamped_pb2.TransformStamped
    admittance_position: _stamped_pb2.TransformStamped
    admittance_acceleration: _stamped_pb2.TwistStamped
    admittance_velocity: _stamped_pb2.TwistStamped
    wrench_base: _stamped_pb2.WrenchStamped
    robot_ref_trans_base_fts: _stamped_pb2.TransformStamped
    joint_names: _containers.RepeatedScalarFieldContainer[str]
    joint_state: _joint_state_pb2.JointState
    def __init__(self, ref_trans_base_fts: _Optional[_Union[_stamped_pb2.TransformStamped, _Mapping]] = ..., selected_axes: _Optional[_Union[_multi_array_msgs_pb2.Float64MultiArray, _Mapping]] = ..., ft_sensor_frame: _Optional[_Union[_stamped_pb2.TransformStamped, _Mapping]] = ..., admittance_position: _Optional[_Union[_stamped_pb2.TransformStamped, _Mapping]] = ..., admittance_acceleration: _Optional[_Union[_stamped_pb2.TwistStamped, _Mapping]] = ..., admittance_velocity: _Optional[_Union[_stamped_pb2.TwistStamped, _Mapping]] = ..., wrench_base: _Optional[_Union[_stamped_pb2.WrenchStamped, _Mapping]] = ..., robot_ref_trans_base_fts: _Optional[_Union[_stamped_pb2.TransformStamped, _Mapping]] = ..., joint_names: _Optional[_Iterable[str]] = ..., joint_state: _Optional[_Union[_joint_state_pb2.JointState, _Mapping]] = ...) -> None: ...
