"""Generated mapper for `geometry_msgs/msg/PolygonInstanceStamped`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon_instance import polygon_instance_to_bus, polygon_instance_to_ros

def polygon_instance_stamped_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import PolygonInstanceStamped as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.polygon.CopyFrom(polygon_instance_to_bus(msg.polygon))
    return bus


def polygon_instance_stamped_to_ros(bus):
    from geometry_msgs.msg import PolygonInstanceStamped as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.polygon = polygon_instance_to_ros(bus.polygon)
    return out


class GeometryMsgsPolygonInstanceStampedMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import PolygonInstanceStamped as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return polygon_instance_stamped_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import PolygonInstanceStamped as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return polygon_instance_stamped_to_ros(bus)
