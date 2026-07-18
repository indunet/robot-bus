from robot_bus.foxglove_msgs.msg.v1 import circle_annotation_pb2 as _circle_annotation_pb2
from robot_bus.foxglove_msgs.msg.v1 import key_value_pair_pb2 as _key_value_pair_pb2
from robot_bus.foxglove_msgs.msg.v1 import points_annotation_pb2 as _points_annotation_pb2
from robot_bus.foxglove_msgs.msg.v1 import text_annotation_pb2 as _text_annotation_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ImageAnnotations(_message.Message):
    __slots__ = ("timestamp", "circles", "points", "texts", "metadata")
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    CIRCLES_FIELD_NUMBER: _ClassVar[int]
    POINTS_FIELD_NUMBER: _ClassVar[int]
    TEXTS_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    circles: _containers.RepeatedCompositeFieldContainer[_circle_annotation_pb2.CircleAnnotation]
    points: _containers.RepeatedCompositeFieldContainer[_points_annotation_pb2.PointsAnnotation]
    texts: _containers.RepeatedCompositeFieldContainer[_text_annotation_pb2.TextAnnotation]
    metadata: _containers.RepeatedCompositeFieldContainer[_key_value_pair_pb2.KeyValuePair]
    def __init__(self, timestamp: _Optional[_Union[_timestamp_pb2.Timestamp, _Mapping]] = ..., circles: _Optional[_Iterable[_Union[_circle_annotation_pb2.CircleAnnotation, _Mapping]]] = ..., points: _Optional[_Iterable[_Union[_points_annotation_pb2.PointsAnnotation, _Mapping]]] = ..., texts: _Optional[_Iterable[_Union[_text_annotation_pb2.TextAnnotation, _Mapping]]] = ..., metadata: _Optional[_Iterable[_Union[_key_value_pair_pb2.KeyValuePair, _Mapping]]] = ...) -> None: ...
