"""Generated mapper for `std_msgs/msg/ColorRGBA`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.std_msgs.msg.v1 import ColorRGBA as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def color_rgba_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from std_msgs.msg import ColorRGBA as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return color_rgba_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return color_rgba_to_ros(bus)
