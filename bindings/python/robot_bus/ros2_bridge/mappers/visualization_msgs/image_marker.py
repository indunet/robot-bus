"""Generated mapper for `visualization_msgs/msg/ImageMarker`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros
from robot_bus.ros2_bridge.mappers.std_msgs.color_rgba import color_rgba_to_bus, color_rgba_to_ros
from robot_bus.ros2_bridge.mappers.builtin_interfaces.duration import duration_to_bus, duration_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.visualization_msgs.msg.v1 import ImageMarker as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def image_marker_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.ns = str(msg.ns)
    bus.id = msg.id
    bus.type = msg.type
    bus.action = msg.action
    bus.position.CopyFrom(point_to_bus(msg.position))
    bus.scale = msg.scale
    bus.outline_color.CopyFrom(color_rgba_to_bus(msg.outline_color))
    bus.filled = msg.filled
    bus.fill_color.CopyFrom(color_rgba_to_bus(msg.fill_color))
    bus.lifetime.CopyFrom(duration_to_bus(msg.lifetime))
    bus.points.extend([point_to_bus(x) for x in msg.points])
    bus.outline_colors.extend([color_rgba_to_bus(x) for x in msg.outline_colors])
    return bus


def image_marker_to_ros(bus):
    from visualization_msgs.msg import ImageMarker as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.ns = str(bus.ns)
    out.id = bus.id
    out.type = bus.type
    out.action = bus.action
    out.position = point_to_ros(bus.position)
    out.scale = bus.scale
    out.outline_color = color_rgba_to_ros(bus.outline_color)
    out.filled = bus.filled
    out.fill_color = color_rgba_to_ros(bus.fill_color)
    out.lifetime = duration_to_ros(bus.lifetime)
    out.points = [point_to_ros(x) for x in bus.points]
    out.outline_colors = [color_rgba_to_ros(x) for x in bus.outline_colors]
    return out


class VisualizationMsgsImageMarkerMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from visualization_msgs.msg import ImageMarker as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return image_marker_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return image_marker_to_ros(bus)
