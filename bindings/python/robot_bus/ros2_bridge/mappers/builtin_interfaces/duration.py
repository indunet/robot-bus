"""Generated mapper for `builtin_interfaces/msg/Duration`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def duration_to_bus(msg):
    from robot_bus.builtin_interfaces.msg.v1 import Duration as BusMsg

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
    def ros_msg_type(self):
        from builtin_interfaces.msg import Duration as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return duration_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.builtin_interfaces.msg.v1 import Duration as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return duration_to_ros(bus)
