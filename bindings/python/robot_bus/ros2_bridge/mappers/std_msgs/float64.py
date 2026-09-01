"""Generated mapper for `std_msgs/msg/Float64`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def float64_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Float64 as BusMsg

    bus = BusMsg()
    bus.data = msg.data
    return bus


def float64_to_ros(bus):
    from std_msgs.msg import Float64 as RosMsg

    out = RosMsg()
    out.data = bus.data
    return out


class StdMsgsFloat64Mapper:
    def ros_msg_type(self):
        from std_msgs.msg import Float64 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return float64_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Float64 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return float64_to_ros(bus)
