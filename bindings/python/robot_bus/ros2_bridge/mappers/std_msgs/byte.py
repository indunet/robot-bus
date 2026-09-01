"""Generated mapper for `std_msgs/msg/Byte`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def byte_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Byte as BusMsg

    bus = BusMsg()
    bus.data = int(msg.data)
    return bus


def byte_to_ros(bus):
    from std_msgs.msg import Byte as RosMsg

    out = RosMsg()
    out.data = int(bus.data)
    return out


class StdMsgsByteMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/Byte"

    def ros_msg_type(self):
        from std_msgs.msg import Byte as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return byte_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Byte as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return byte_to_ros(bus)
