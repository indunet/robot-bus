from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from robot_bus.geometry_msgs.msg.v1 import pose_pb2 as _pose_pb2
from robot_bus.geometry_msgs.msg.v1 import quaternion_pb2 as _quaternion_pb2
from robot_bus.geometry_msgs.msg.v1 import point_pb2 as _point_pb2
from robot_bus.visualization_msgs.msg.v1 import marker_pb2 as _marker_pb2
from robot_bus.visualization_msgs.msg.v1 import menu_entry_pb2 as _menu_entry_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class InteractiveMarkerControl(_message.Message):
    __slots__ = ("name", "orientation", "orientation_mode", "interaction_mode", "always_visible", "markers", "independent_marker_orientation", "description")
    NAME_FIELD_NUMBER: _ClassVar[int]
    ORIENTATION_FIELD_NUMBER: _ClassVar[int]
    ORIENTATION_MODE_FIELD_NUMBER: _ClassVar[int]
    INTERACTION_MODE_FIELD_NUMBER: _ClassVar[int]
    ALWAYS_VISIBLE_FIELD_NUMBER: _ClassVar[int]
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    INDEPENDENT_MARKER_ORIENTATION_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    name: str
    orientation: _quaternion_pb2.Quaternion
    orientation_mode: int
    interaction_mode: int
    always_visible: bool
    markers: _containers.RepeatedCompositeFieldContainer[_marker_pb2.Marker]
    independent_marker_orientation: bool
    description: str
    def __init__(self, name: _Optional[str] = ..., orientation: _Optional[_Union[_quaternion_pb2.Quaternion, _Mapping]] = ..., orientation_mode: _Optional[int] = ..., interaction_mode: _Optional[int] = ..., always_visible: bool = ..., markers: _Optional[_Iterable[_Union[_marker_pb2.Marker, _Mapping]]] = ..., independent_marker_orientation: bool = ..., description: _Optional[str] = ...) -> None: ...

class InteractiveMarker(_message.Message):
    __slots__ = ("header", "pose", "name", "description", "scale", "menu_entries", "controls")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    SCALE_FIELD_NUMBER: _ClassVar[int]
    MENU_ENTRIES_FIELD_NUMBER: _ClassVar[int]
    CONTROLS_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    pose: _pose_pb2.Pose
    name: str
    description: str
    scale: float
    menu_entries: _containers.RepeatedCompositeFieldContainer[_menu_entry_pb2.MenuEntry]
    controls: _containers.RepeatedCompositeFieldContainer[InteractiveMarkerControl]
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., name: _Optional[str] = ..., description: _Optional[str] = ..., scale: _Optional[float] = ..., menu_entries: _Optional[_Iterable[_Union[_menu_entry_pb2.MenuEntry, _Mapping]]] = ..., controls: _Optional[_Iterable[_Union[InteractiveMarkerControl, _Mapping]]] = ...) -> None: ...

class InteractiveMarkerPose(_message.Message):
    __slots__ = ("header", "pose", "name")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    NAME_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    pose: _pose_pb2.Pose
    name: str
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., name: _Optional[str] = ...) -> None: ...

class InteractiveMarkerInit(_message.Message):
    __slots__ = ("server_id", "seq_num", "markers")
    SERVER_ID_FIELD_NUMBER: _ClassVar[int]
    SEQ_NUM_FIELD_NUMBER: _ClassVar[int]
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    server_id: str
    seq_num: int
    markers: _containers.RepeatedCompositeFieldContainer[InteractiveMarker]
    def __init__(self, server_id: _Optional[str] = ..., seq_num: _Optional[int] = ..., markers: _Optional[_Iterable[_Union[InteractiveMarker, _Mapping]]] = ...) -> None: ...

class InteractiveMarkerUpdate(_message.Message):
    __slots__ = ("server_id", "seq_num", "type", "markers", "poses", "erases")
    SERVER_ID_FIELD_NUMBER: _ClassVar[int]
    SEQ_NUM_FIELD_NUMBER: _ClassVar[int]
    TYPE_FIELD_NUMBER: _ClassVar[int]
    MARKERS_FIELD_NUMBER: _ClassVar[int]
    POSES_FIELD_NUMBER: _ClassVar[int]
    ERASES_FIELD_NUMBER: _ClassVar[int]
    server_id: str
    seq_num: int
    type: int
    markers: _containers.RepeatedCompositeFieldContainer[InteractiveMarker]
    poses: _containers.RepeatedCompositeFieldContainer[InteractiveMarkerPose]
    erases: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, server_id: _Optional[str] = ..., seq_num: _Optional[int] = ..., type: _Optional[int] = ..., markers: _Optional[_Iterable[_Union[InteractiveMarker, _Mapping]]] = ..., poses: _Optional[_Iterable[_Union[InteractiveMarkerPose, _Mapping]]] = ..., erases: _Optional[_Iterable[str]] = ...) -> None: ...

class InteractiveMarkerFeedback(_message.Message):
    __slots__ = ("header", "client_id", "marker_name", "control_name", "event_type", "pose", "menu_entry_id", "mouse_point", "mouse_point_valid")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    CLIENT_ID_FIELD_NUMBER: _ClassVar[int]
    MARKER_NAME_FIELD_NUMBER: _ClassVar[int]
    CONTROL_NAME_FIELD_NUMBER: _ClassVar[int]
    EVENT_TYPE_FIELD_NUMBER: _ClassVar[int]
    POSE_FIELD_NUMBER: _ClassVar[int]
    MENU_ENTRY_ID_FIELD_NUMBER: _ClassVar[int]
    MOUSE_POINT_FIELD_NUMBER: _ClassVar[int]
    MOUSE_POINT_VALID_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    client_id: str
    marker_name: str
    control_name: str
    event_type: int
    pose: _pose_pb2.Pose
    menu_entry_id: int
    mouse_point: _point_pb2.Point
    mouse_point_valid: bool
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., client_id: _Optional[str] = ..., marker_name: _Optional[str] = ..., control_name: _Optional[str] = ..., event_type: _Optional[int] = ..., pose: _Optional[_Union[_pose_pb2.Pose, _Mapping]] = ..., menu_entry_id: _Optional[int] = ..., mouse_point: _Optional[_Union[_point_pb2.Point, _Mapping]] = ..., mouse_point_valid: bool = ...) -> None: ...
