from robot_bus.foxglove_msgs.msg.v1 import color_pb2 as _color_pb2
from robot_bus.foxglove_msgs.msg.v1 import key_value_pair_pb2 as _key_value_pair_pb2
from robot_bus.foxglove_msgs.msg.v1 import vector3_pb2 as _vector3_pb2
from google.protobuf import timestamp_pb2 as _timestamp_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class LocationFix(_message.Message):
    __slots__ = ("timestamp", "frame_id", "latitude", "longitude", "altitude", "position_covariance", "position_covariance_type", "heading", "velocity", "color", "metadata")
    class PositionCovarianceType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
        __slots__ = ()
        UNKNOWN: _ClassVar[LocationFix.PositionCovarianceType]
        APPROXIMATED: _ClassVar[LocationFix.PositionCovarianceType]
        DIAGONAL_KNOWN: _ClassVar[LocationFix.PositionCovarianceType]
        KNOWN: _ClassVar[LocationFix.PositionCovarianceType]
    UNKNOWN: LocationFix.PositionCovarianceType
    APPROXIMATED: LocationFix.PositionCovarianceType
    DIAGONAL_KNOWN: LocationFix.PositionCovarianceType
    KNOWN: LocationFix.PositionCovarianceType
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    FRAME_ID_FIELD_NUMBER: _ClassVar[int]
    LATITUDE_FIELD_NUMBER: _ClassVar[int]
    LONGITUDE_FIELD_NUMBER: _ClassVar[int]
    ALTITUDE_FIELD_NUMBER: _ClassVar[int]
    POSITION_COVARIANCE_FIELD_NUMBER: _ClassVar[int]
    POSITION_COVARIANCE_TYPE_FIELD_NUMBER: _ClassVar[int]
    HEADING_FIELD_NUMBER: _ClassVar[int]
    VELOCITY_FIELD_NUMBER: _ClassVar[int]
    COLOR_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    timestamp: _timestamp_pb2.Timestamp
    frame_id: str
    latitude: float
    longitude: float
    altitude: float
    position_covariance: _containers.RepeatedScalarFieldContainer[float]
    position_covariance_type: LocationFix.PositionCovarianceType
    heading: float
    velocity: _vector3_pb2.Vector3
    color: _color_pb2.Color
    metadata: _containers.RepeatedCompositeFieldContainer[_key_value_pair_pb2.KeyValuePair]
    def __init__(self, timestamp: _Optional[_Union[_timestamp_pb2.Timestamp, _Mapping]] = ..., frame_id: _Optional[str] = ..., latitude: _Optional[float] = ..., longitude: _Optional[float] = ..., altitude: _Optional[float] = ..., position_covariance: _Optional[_Iterable[float]] = ..., position_covariance_type: _Optional[_Union[LocationFix.PositionCovarianceType, str]] = ..., heading: _Optional[float] = ..., velocity: _Optional[_Union[_vector3_pb2.Vector3, _Mapping]] = ..., color: _Optional[_Union[_color_pb2.Color, _Mapping]] = ..., metadata: _Optional[_Iterable[_Union[_key_value_pair_pb2.KeyValuePair, _Mapping]]] = ...) -> None: ...
