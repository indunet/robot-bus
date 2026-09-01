"""Generated mapper for `foxglove_msgs/msg/Point3`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def point3_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import Point3 as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    bus.z = msg.z
    return bus


def point3_to_ros(bus):
    from foxglove_msgs.msg import Point3 as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    out.z = bus.z
    return out


class FoxgloveMsgsPoint3Mapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/Point3"

    def ros_msg_type(self):
        from foxglove_msgs.msg import Point3 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point3_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import Point3 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point3_to_ros(bus)
