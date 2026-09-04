"""Generated mapper for `foxglove_msgs/msg/Color`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.foxglove_msgs.msg.v1 import Color as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def color_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from foxglove_msgs.msg import Color as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return color_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return color_to_ros(bus)
