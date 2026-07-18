from robot_bus.foxglove_msgs.msg.v1 import scene_entity_pb2 as _scene_entity_pb2
from robot_bus.foxglove_msgs.msg.v1 import scene_entity_deletion_pb2 as _scene_entity_deletion_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SceneUpdate(_message.Message):
    __slots__ = ("deletions", "entities")
    DELETIONS_FIELD_NUMBER: _ClassVar[int]
    ENTITIES_FIELD_NUMBER: _ClassVar[int]
    deletions: _containers.RepeatedCompositeFieldContainer[_scene_entity_deletion_pb2.SceneEntityDeletion]
    entities: _containers.RepeatedCompositeFieldContainer[_scene_entity_pb2.SceneEntity]
    def __init__(self, deletions: _Optional[_Iterable[_Union[_scene_entity_deletion_pb2.SceneEntityDeletion, _Mapping]]] = ..., entities: _Optional[_Iterable[_Union[_scene_entity_pb2.SceneEntity, _Mapping]]] = ...) -> None: ...
