from robot_bus.foxglove_msgs.msg.v1 import frame_transform_pb2 as _frame_transform_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class FrameTransforms(_message.Message):
    __slots__ = ("transforms",)
    TRANSFORMS_FIELD_NUMBER: _ClassVar[int]
    transforms: _containers.RepeatedCompositeFieldContainer[_frame_transform_pb2.FrameTransform]
    def __init__(self, transforms: _Optional[_Iterable[_Union[_frame_transform_pb2.FrameTransform, _Mapping]]] = ...) -> None: ...
