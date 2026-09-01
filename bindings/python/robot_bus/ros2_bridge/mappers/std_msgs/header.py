"""Generated mapper for `std_msgs/msg/Header`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import time_to_bus, time_to_ros

def header_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Header as BusMsg

    bus = BusMsg()
    bus.stamp.CopyFrom(time_to_bus(msg.stamp))
    bus.frame_id = str(msg.frame_id)
    return bus


def header_to_ros(bus):
    from std_msgs.msg import Header as RosMsg

    out = RosMsg()
    out.stamp = time_to_ros(bus.stamp)
    out.frame_id = str(bus.frame_id)
    return out


class StdMsgsHeaderMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/Header"

    def ros_msg_type(self):
        from std_msgs.msg import Header as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return header_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Header as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return header_to_ros(bus)
