"""Generated mapper for `foxglove_msgs/msg/CircleAnnotation`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point2 import point2_to_bus, point2_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import CircleAnnotation as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def circle_annotation_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.position.CopyFrom(point2_to_bus(msg.position))
    bus.diameter = msg.diameter
    bus.thickness = msg.thickness
    bus.fill_color.CopyFrom(color_to_bus(msg.fill_color))
    bus.outline_color.CopyFrom(color_to_bus(msg.outline_color))
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    return bus


def circle_annotation_to_ros(bus):
    from foxglove_msgs.msg import CircleAnnotation as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.position = point2_to_ros(bus.position)
    out.diameter = bus.diameter
    out.thickness = bus.thickness
    out.fill_color = color_to_ros(bus.fill_color)
    out.outline_color = color_to_ros(bus.outline_color)
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    return out


class FoxgloveMsgsCircleAnnotationMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import CircleAnnotation as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return circle_annotation_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return circle_annotation_to_ros(bus)
