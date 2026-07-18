from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class GeoJSON(_message.Message):
    __slots__ = ("geojson",)
    GEOJSON_FIELD_NUMBER: _ClassVar[int]
    geojson: str
    def __init__(self, geojson: _Optional[str] = ...) -> None: ...
