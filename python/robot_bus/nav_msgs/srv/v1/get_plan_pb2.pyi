from robot_bus.geometry_msgs.msg.v1 import stamped_pb2 as _stamped_pb2
from robot_bus.nav_msgs.msg.v1 import path_pb2 as _path_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class GetPlanRequest(_message.Message):
    __slots__ = ("start", "goal", "tolerance")
    START_FIELD_NUMBER: _ClassVar[int]
    GOAL_FIELD_NUMBER: _ClassVar[int]
    TOLERANCE_FIELD_NUMBER: _ClassVar[int]
    start: _stamped_pb2.PoseStamped
    goal: _stamped_pb2.PoseStamped
    tolerance: float
    def __init__(self, start: _Optional[_Union[_stamped_pb2.PoseStamped, _Mapping]] = ..., goal: _Optional[_Union[_stamped_pb2.PoseStamped, _Mapping]] = ..., tolerance: _Optional[float] = ...) -> None: ...

class GetPlanResponse(_message.Message):
    __slots__ = ("plan",)
    PLAN_FIELD_NUMBER: _ClassVar[int]
    plan: _path_pb2.Path
    def __init__(self, plan: _Optional[_Union[_path_pb2.Path, _Mapping]] = ...) -> None: ...
