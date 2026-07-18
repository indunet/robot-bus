from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class Log(_message.Message):
    __slots__ = ("timestamp", "level", "message", "name", "file", "line")
    class Level(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
        __slots__ = ()
        UNKNOWN: _ClassVar[Log.Level]
        DEBUG: _ClassVar[Log.Level]
        INFO: _ClassVar[Log.Level]
        WARNING: _ClassVar[Log.Level]
        ERROR: _ClassVar[Log.Level]
        FATAL: _ClassVar[Log.Level]
    UNKNOWN: Log.Level
    DEBUG: Log.Level
    INFO: Log.Level
    WARNING: Log.Level
    ERROR: Log.Level
    FATAL: Log.Level
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    LEVEL_FIELD_NUMBER: _ClassVar[int]
    MESSAGE_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    FILE_FIELD_NUMBER: _ClassVar[int]
    LINE_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    level: Log.Level
    message: str
    name: str
    file: str
    line: int
    def __init__(self, timestamp: _Optional[_Union[_timestamp_pb2.Timestamp, _Mapping]] = ..., level: _Optional[_Union[Log.Level, str]] = ..., message: _Optional[str] = ..., name: _Optional[str] = ..., file: _Optional[str] = ..., line: _Optional[int] = ...) -> None: ...
