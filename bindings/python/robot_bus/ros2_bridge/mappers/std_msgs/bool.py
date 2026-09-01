"""Generated mapper for `std_msgs/msg/Bool`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def bool_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Bool as BusMsg

    bus = BusMsg()
    bus.data = msg.data
    return bus


def bool_to_ros(bus):
    from std_msgs.msg import Bool as RosMsg

    out = RosMsg()
    out.data = bus.data
    return out


class StdMsgsBoolMapper:
    def ros_msg_type(self):
        from std_msgs.msg import Bool as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return bool_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Bool as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return bool_to_ros(bus)
