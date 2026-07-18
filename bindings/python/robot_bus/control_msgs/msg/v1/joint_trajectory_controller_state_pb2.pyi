from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.trajectory_msgs.msg.v1 import joint_trajectory_pb2 as _joint_trajectory_pb2
from robot_bus.trajectory_msgs.msg.v1 import multi_dof_joint_trajectory_pb2 as _multi_dof_joint_trajectory_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class JointTrajectoryControllerState(_message.Message):
    __slots__ = ("header", "joint_names", "reference", "feedback", "error", "output", "multi_dof_joint_names", "multi_dof_reference", "multi_dof_feedback", "multi_dof_error", "multi_dof_output")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_FIELD_NUMBER: _ClassVar[int]
    FEEDBACK_FIELD_NUMBER: _ClassVar[int]
    ERROR_FIELD_NUMBER: _ClassVar[int]
    OUTPUT_FIELD_NUMBER: _ClassVar[int]
    MULTI_DOF_JOINT_NAMES_FIELD_NUMBER: _ClassVar[int]
    MULTI_DOF_REFERENCE_FIELD_NUMBER: _ClassVar[int]
    MULTI_DOF_FEEDBACK_FIELD_NUMBER: _ClassVar[int]
    MULTI_DOF_ERROR_FIELD_NUMBER: _ClassVar[int]
    MULTI_DOF_OUTPUT_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    joint_names: _containers.RepeatedScalarFieldContainer[str]
    reference: _joint_trajectory_pb2.JointTrajectoryPoint
    feedback: _joint_trajectory_pb2.JointTrajectoryPoint
    error: _joint_trajectory_pb2.JointTrajectoryPoint
    output: _joint_trajectory_pb2.JointTrajectoryPoint
    multi_dof_joint_names: _containers.RepeatedScalarFieldContainer[str]
    multi_dof_reference: _multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint
    multi_dof_feedback: _multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint
    multi_dof_error: _multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint
    multi_dof_output: _multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., joint_names: _Optional[_Iterable[str]] = ..., reference: _Optional[_Union[_joint_trajectory_pb2.JointTrajectoryPoint, _Mapping]] = ..., feedback: _Optional[_Union[_joint_trajectory_pb2.JointTrajectoryPoint, _Mapping]] = ..., error: _Optional[_Union[_joint_trajectory_pb2.JointTrajectoryPoint, _Mapping]] = ..., output: _Optional[_Union[_joint_trajectory_pb2.JointTrajectoryPoint, _Mapping]] = ..., multi_dof_joint_names: _Optional[_Iterable[str]] = ..., multi_dof_reference: _Optional[_Union[_multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint, _Mapping]] = ..., multi_dof_feedback: _Optional[_Union[_multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint, _Mapping]] = ..., multi_dof_error: _Optional[_Union[_multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint, _Mapping]] = ..., multi_dof_output: _Optional[_Union[_multi_dof_joint_trajectory_pb2.MultiDOFJointTrajectoryPoint, _Mapping]] = ...) -> None: ...
