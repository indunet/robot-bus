"""Generated mapper for `geometry_msgs/msg/PointStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import point_to_bus, point_to_ros

def point_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import PointStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.point.CopyFrom(point_to_bus(msg.point))
    return bus


def point_stamped_to_ros(bus):
    from geometry_msgs.msg import PointStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.point = point_to_ros(bus.point)
    return out


class GeometryMsgsPointStampedMapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/PointStamped"

    def ros_msg_type(self):
        from geometry_msgs.msg import PointStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import PointStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_stamped_to_ros(bus)
