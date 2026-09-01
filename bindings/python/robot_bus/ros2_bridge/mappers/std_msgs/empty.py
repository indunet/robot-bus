"""Generated mapper for `std_msgs/msg/Empty`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def empty_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Empty as BusMsg

    bus = BusMsg()
    pass
    return bus


def empty_to_ros(bus):
    from std_msgs.msg import Empty as RosMsg

    out = RosMsg()
    pass
    return out


class StdMsgsEmptyMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/Empty"

    def ros_msg_type(self):
        from std_msgs.msg import Empty as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return empty_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Empty as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return empty_to_ros(bus)
