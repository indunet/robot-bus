from robot_bus.foxglove_msgs.msg.v1 import quaternion_pb2 as _quaternion_pb2
from robot_bus.foxglove_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Pose(_message.Message):
    __slots__ = ("position", "orientation")
    POSITION_FIELD_NUMBER: _ClassVar[int]
    ORIENTATION_FIELD_NUMBER: _ClassVar[int]
    position: _vector3_pb2.Vector3
    orientation: _quaternion_pb2.Quaternion
    def __init__(self, position: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., orientation: _Optional[_Union[_quaternion_pb2.Quaternion, _Mapping]] = ...) -> None: ...
