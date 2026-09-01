"""Generated mapper for `std_msgs/msg/ColorRGBA`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def color_rgba_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import ColorRGBA as BusMsg

    bus = BusMsg()
    bus.r = msg.r
    bus.g = msg.g
    bus.b = msg.b
    bus.a = msg.a
    return bus


def color_rgba_to_ros(bus):
    from std_msgs.msg import ColorRGBA as RosMsg

    out = RosMsg()
    out.r = bus.r
    out.g = bus.g
    out.b = bus.b
    out.a = bus.a
    return out


class StdMsgsColorRgbaMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/ColorRGBA"

    def ros_msg_type(self):
        from std_msgs.msg import ColorRGBA as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return color_rgba_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import ColorRGBA as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return color_rgba_to_ros(bus)
