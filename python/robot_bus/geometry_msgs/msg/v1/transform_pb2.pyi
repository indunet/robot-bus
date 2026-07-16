from robot_bus.geometry_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from robot_bus.geometry_msgs.msg.v1 import quaternion_pb2 as _quaternion_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Transform(_message.Message):
    __slots__ = ("translation", "rotation")
    TRANSLATION_FIELD_NUMBER: _ClassVar[int]
    ROTATION_FIELD_NUMBER: _ClassVar[int]
    translation: _vector3_pb2.Vector3
    rotation: _quaternion_pb2.Quaternion
    def __init__(self, translation: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., rotation: _Optional[_Union[_quaternion_pb2.Quaternion, _Mapping]] = ...) -> None: ...
