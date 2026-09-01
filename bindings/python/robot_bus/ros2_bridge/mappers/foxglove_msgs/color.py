"""Generated mapper for `foxglove_msgs/msg/Color`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def color_to_bus(msg):
    from robot_bus.foxglove_msgs.msg.v1 import Color as BusMsg

    bus = BusMsg()
    bus.r = msg.r
    bus.g = msg.g
    bus.b = msg.b
    bus.a = msg.a
    return bus


def color_to_ros(bus):
    from foxglove_msgs.msg import Color as RosMsg

    out = RosMsg()
    out.r = bus.r
    out.g = bus.g
    out.b = bus.b
    out.a = bus.a
    return out


class FoxgloveMsgsColorMapper:
    def ros_msg_type(self):
        from foxglove_msgs.msg import Color as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return color_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.foxglove_msgs.msg.v1 import Color as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return color_to_ros(bus)
