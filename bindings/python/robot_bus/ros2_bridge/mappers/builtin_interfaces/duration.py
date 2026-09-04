"""Generated mapper for `builtin_interfaces/msg/Duration`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.builtin_interfaces.msg.v1 import Duration as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def duration_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.sec = msg.sec
    bus.nanosec = msg.nanosec
    return bus


def duration_to_ros(bus):
    from builtin_interfaces.msg import Duration as RosMsg

    out = RosMsg()
    out.sec = bus.sec
    out.nanosec = bus.nanosec
    return out


class BuiltinInterfacesDurationMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from builtin_interfaces.msg import Duration as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return duration_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return duration_to_ros(bus)
