"""Generated mapper for `foxglove_msgs/msg/PointsAnnotation`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point2 import point2_to_bus, point2_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import color_to_bus, color_to_ros
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import key_value_pair_to_bus, key_value_pair_to_ros

def points_annotation_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import PointsAnnotation as BusMsg

    bus = BusMsg()
    bus.timestamp = _convert.time_to_timestamp(msg.timestamp)
    bus.type = int(msg.type)
    bus.points.extend([point2_to_bus(x) for x in msg.points])
    bus.outline_color.CopyFrom(color_to_bus(msg.outline_color))
    bus.outline_colors.extend([color_to_bus(x) for x in msg.outline_colors])
    bus.fill_color.CopyFrom(color_to_bus(msg.fill_color))
    bus.thickness = msg.thickness
    bus.metadata.extend([key_value_pair_to_bus(x) for x in msg.metadata])
    return bus


def points_annotation_to_ros(bus):
    from foxglove_msgs.msg import PointsAnnotation as RosMsg

    out = RosMsg()
    out.timestamp = _convert.timestamp_to_time(bus.timestamp)
    out.type = int(bus.type)
    out.points = [point2_to_ros(x) for x in bus.points]
    out.outline_color = color_to_ros(bus.outline_color)
    out.outline_colors = [color_to_ros(x) for x in bus.outline_colors]
    out.fill_color = color_to_ros(bus.fill_color)
    out.thickness = bus.thickness
    out.metadata = [key_value_pair_to_ros(x) for x in bus.metadata]
    return out


class FoxgloveMsgsPointsAnnotationMapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/PointsAnnotation"

    def ros_msg_type(self):
        from foxglove_msgs.msg import PointsAnnotation as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return points_annotation_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import PointsAnnotation as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return points_annotation_to_ros(bus)
