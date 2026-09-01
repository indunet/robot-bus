"""Generated mapper for `std_msgs/msg/Int8`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def int8_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Int8 as BusMsg

    bus = BusMsg()
    bus.data = int(msg.data)
    return bus


def int8_to_ros(bus):
    from std_msgs.msg import Int8 as RosMsg

    out = RosMsg()
    out.data = int(bus.data)
    return out


class StdMsgsInt8Mapper:
    def ros_msg_type(self):
        from std_msgs.msg import Int8 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return int8_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Int8 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return int8_to_ros(bus)
