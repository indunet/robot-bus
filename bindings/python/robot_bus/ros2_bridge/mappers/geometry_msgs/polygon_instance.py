"""Generated mapper for `geometry_msgs/msg/PolygonInstance`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon import polygon_to_bus, polygon_to_ros

def polygon_instance_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import PolygonInstance as BusMsg

    bus = BusMsg()
    bus.polygon.CopyFrom(polygon_to_bus(msg.polygon))
    bus.id = msg.id
    return bus


def polygon_instance_to_ros(bus):
    from geometry_msgs.msg import PolygonInstance as RosMsg

    out = RosMsg()
    out.polygon = polygon_to_ros(bus.polygon)
    out.id = bus.id
    return out


class GeometryMsgsPolygonInstanceMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import PolygonInstance as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return polygon_instance_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import PolygonInstance as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return polygon_instance_to_ros(bus)
