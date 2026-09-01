"""Generated mapper for `std_msgs/msg/Int16`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def int16_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Int16 as BusMsg

    bus = BusMsg()
    bus.data = int(msg.data)
    return bus


def int16_to_ros(bus):
    from std_msgs.msg import Int16 as RosMsg

    out = RosMsg()
    out.data = int(bus.data)
    return out


class StdMsgsInt16Mapper:
    def type_name(self) -> str:
        return "std_msgs/msg/Int16"

    def ros_msg_type(self):
        from std_msgs.msg import Int16 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return int16_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Int16 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return int16_to_ros(bus)
