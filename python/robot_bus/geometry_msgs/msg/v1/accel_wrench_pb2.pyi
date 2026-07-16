from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Accel(_message.Message):
    __slots__ = ("linear", "angular")
    LINEAR_FIELD_NUMBER: _ClassVar[int]
    ANGULAR_FIELD_NUMBER: _ClassVar[int]
    linear: _vector3_pb2.Vector3
    angular: _vector3_pb2.Vector3
    def __init__(self, linear: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., angular: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ...) -> None: ...

class Wrench(_message.Message):
    __slots__ = ("force", "torque")
    FORCE_FIELD_NUMBER: _ClassVar[int]
    TORQUE_FIELD_NUMBER: _ClassVar[int]
    force: _vector3_pb2.Vector3
    torque: _vector3_pb2.Vector3
    def __init__(self, force: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., torque: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ...) -> None: ...
