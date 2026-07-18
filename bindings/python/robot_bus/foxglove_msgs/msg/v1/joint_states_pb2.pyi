import datetime

from robot_bus.foxglove_msgs.msg.v1 import joint_state_pb2 as _joint_state_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class JointStates(_message.Message):
    __slots__ = ("timestamp", "joints")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    JOINTS_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    joints: _containers.RepeatedCompositeFieldContainer[_joint_state_pb2.JointState]
    def __init__(self, timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., joints: _Optional[_Iterable[_Union[_joint_state_pb2.JointState, _Mapping]]] = ...) -> None: ...
