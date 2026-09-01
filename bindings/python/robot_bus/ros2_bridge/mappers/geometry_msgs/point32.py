"""Generated mapper for `geometry_msgs/msg/Point32`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def point32_to_bus(msg):
    from robot_bus.geometry_msgs.msg.v1 import Point32 as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    bus.z = msg.z
    return bus


def point32_to_ros(bus):
    from geometry_msgs.msg import Point32 as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    out.z = bus.z
    return out


class GeometryMsgsPoint32Mapper:
    def type_name(self) -> str:
        return "geometry_msgs/msg/Point32"

    def ros_msg_type(self):
        from geometry_msgs.msg import Point32 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point32_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.geometry_msgs.msg.v1 import Point32 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point32_to_ros(bus)
