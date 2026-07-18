from robot_bus.builtin_interfaces.msg.v1 import time_pb2 as _time_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class BehaviorTreeStatusChange(_message.Message):
    __slots__ = ("timestamp", "node_name", "previous_status", "current_status")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    NODE_NAME_FIELD_NUMBER: _ClassVar[int]
    PREVIOUS_STATUS_FIELD_NUMBER: _ClassVar[int]
    CURRENT_STATUS_FIELD_NUMBER: _ClassVar[int]
    timestamp: _time_pb2.Time
    node_name: str
    previous_status: str
    current_status: str
    def __init__(self, timestamp: _Optional[_Union[_time_pb2.Time, _Mapping]] = ..., node_name: _Optional[str] = ..., previous_status: _Optional[str] = ..., current_status: _Optional[str] = ...) -> None: ...

class BehaviorTreeLog(_message.Message):
    __slots__ = ("timestamp", "event_log")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    EVENT_LOG_FIELD_NUMBER: _ClassVar[int]
    timestamp: _time_pb2.Time
    event_log: _containers.RepeatedCompositeFieldContainer[BehaviorTreeStatusChange]
    def __init__(self, timestamp: _Optional[_Union[_time_pb2.Time, _Mapping]] = ..., event_log: _Optional[_Iterable[_Union[BehaviorTreeStatusChange, _Mapping]]] = ...) -> None: ...
