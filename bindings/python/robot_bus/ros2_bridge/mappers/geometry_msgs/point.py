"""Generated mapper for `geometry_msgs/msg/Point`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def point_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Point as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    bus.z = msg.z
    return bus


def point_to_ros(bus):
    from geometry_msgs.msg import Point as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    out.z = bus.z
    return out


class GeometryMsgsPointMapper:
    def ros_msg_type(self):
        from geometry_msgs.msg import Point as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Point as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point_to_ros(bus)
