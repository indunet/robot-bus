"""Generated mapper for `foxglove_msgs/msg/Point2`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def point2_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import Point2 as BusMsg

    bus = BusMsg()
    bus.x = msg.x
    bus.y = msg.y
    return bus


def point2_to_ros(bus):
    from foxglove_msgs.msg import Point2 as RosMsg

    out = RosMsg()
    out.x = bus.x
    out.y = bus.y
    return out


class FoxgloveMsgsPoint2Mapper:
    def type_name(self) -> str:
        return "foxglove_msgs/msg/Point2"

    def ros_msg_type(self):
        from foxglove_msgs.msg import Point2 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return point2_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import Point2 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return point2_to_ros(bus)
