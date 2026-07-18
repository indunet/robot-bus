import datetime

from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class SceneEntityDeletion(_message.Message):
    __slots__ = ("timestamp", "type", "id")
    class Type(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
        __slots__ = ()
        MATCHING_ID: _ClassVar[SceneEntityDeletion.Type]
        ALL: _ClassVar[SceneEntityDeletion.Type]
    MATCHING_ID: SceneEntityDeletion.Type
    ALL: SceneEntityDeletion.Type
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    type: SceneEntityDeletion.Type
    id: str
    def __init__(self, timestamp: _Optional[_Union[datetime.datetime, _timestamp_pb2.Timestamp, _Mapping]] = ..., type: _Optional[_Union[SceneEntityDeletion.Type, str]] = ..., id: _Optional[str] = ...) -> None: ...
