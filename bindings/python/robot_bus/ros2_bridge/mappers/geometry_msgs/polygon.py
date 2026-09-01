"""Generated mapper for `geometry_msgs/msg/Polygon`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.point32 import point32_to_bus, point32_to_ros

def polygon_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Polygon as BusMsg

    bus = BusMsg()
    bus.points.extend([point32_to_bus(x) for x in msg.points])
    return bus


def polygon_to_ros(bus):
    from geometry_msgs.msg import Polygon as RosMsg

    out = RosMsg()
    out.points = [point32_to_ros(x) for x in bus.points]
    return out


class GeometryMsgsPolygonMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import Polygon as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return polygon_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Polygon as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return polygon_to_ros(bus)
