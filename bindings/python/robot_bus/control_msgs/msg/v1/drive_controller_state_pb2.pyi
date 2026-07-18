from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import twist_pb2 as _twist_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class MecanumDriveControllerState(_message.Message):
    __slots__ = ("header", "front_left_wheel_velocity", "front_right_wheel_velocity", "back_left_wheel_velocity", "back_right_wheel_velocity", "reference_velocity")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    FRONT_LEFT_WHEEL_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    FRONT_RIGHT_WHEEL_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    BACK_LEFT_WHEEL_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    BACK_RIGHT_WHEEL_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    REFERENCE_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    front_left_wheel_velocity: float
    front_right_wheel_velocity: float
    back_left_wheel_velocity: float
    back_right_wheel_velocity: float
    reference_velocity: _twist_pb2.Twist
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., front_left_wheel_velocity: _Optional[float] = ..., front_right_wheel_velocity: _Optional[float] = ..., back_left_wheel_velocity: _Optional[float] = ..., back_right_wheel_velocity: _Optional[float] = ..., reference_velocity: _Optional[_Union[_twist_pb2.Twist, _Mapping]] = ...) -> None: ...

class SteeringControllerStatus(_message.Message):
    __slots__ = ("header", "traction_wheels_position", "traction_wheels_velocity", "steer_positions", "linear_velocity_command", "steering_angle_command")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    TRACTION_WHEELS_POSITION_FIELD_NUMBER: _ClassVar[int]
    TRACTION_WHEELS_VELOCITY_FIELD_NUMBER: _ClassVar[int]
    STEER_POSITIONS_FIELD_NUMBER: _ClassVar[int]
    LINEAR_VELOCITY_COMMAND_FIELD_NUMBER: _ClassVar[int]
    STEERING_ANGLE_COMMAND_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    traction_wheels_position: _containers.RepeatedScalarFieldContainer[float]
    traction_wheels_velocity: _containers.RepeatedScalarFieldContainer[float]
    steer_positions: _containers.RepeatedScalarFieldContainer[float]
    linear_velocity_command: _containers.RepeatedScalarFieldContainer[float]
    steering_angle_command: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., traction_wheels_position: _Optional[_Iterable[float]] = ..., traction_wheels_velocity: _Optional[_Iterable[float]] = ..., steer_positions: _Optional[_Iterable[float]] = ..., linear_velocity_command: _Optional[_Iterable[float]] = ..., steering_angle_command: _Optional[_Iterable[float]] = ...) -> None: ...
