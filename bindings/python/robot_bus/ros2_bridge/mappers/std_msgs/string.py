"""Generated mapper for `std_msgs/msg/String`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def string_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import String as BusMsg

    bus = BusMsg()
    bus.data = str(msg.data)
    return bus


def string_to_ros(bus):
    from std_msgs.msg import String as RosMsg

    out = RosMsg()
    out.data = str(bus.data)
    return out


class StdMsgsStringMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/String"

    def ros_msg_type(self):
        from std_msgs.msg import String as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return string_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import String as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return string_to_ros(bus)
