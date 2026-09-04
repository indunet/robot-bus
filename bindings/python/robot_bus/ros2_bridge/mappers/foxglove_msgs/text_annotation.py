"""Generated mapper for `foxglove_msgs/msg/TextAnnotation`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point2 import point2_to_bus, point2_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import TextAnnotation as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def text_annotation_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.position.CopyFrom(point2_to_bus(msg.position))
    bus.text = str(msg.text)
    bus.font_size = msg.font_size
    bus.text_color.CopyFrom(color_to_bus(msg.text_color))
    bus.background_color.CopyFrom(color_to_bus(msg.background_color))
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    return bus


def text_annotation_to_ros(bus):
    from foxglove_msgs.msg import TextAnnotation as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.position = point2_to_ros(bus.position)
    out.text = str(bus.text)
    out.font_size = bus.font_size
    out.text_color = color_to_ros(bus.text_color)
    out.background_color = color_to_ros(bus.background_color)
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    return out


class FoxgloveMsgsTextAnnotationMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import TextAnnotation as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return text_annotation_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return text_annotation_to_ros(bus)
