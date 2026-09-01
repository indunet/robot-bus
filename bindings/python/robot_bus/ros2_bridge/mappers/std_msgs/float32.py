"""Generated mapper for `std_msgs/msg/Float32`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def float32_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import Float32 as BusMsg

    bus = BusMsg()
    bus.data = msg.data
    return bus


def float32_to_ros(bus):
    from std_msgs.msg import Float32 as RosMsg

    out = RosMsg()
    out.data = bus.data
    return out


class StdMsgsFloat32Mapper:
    def type_name(self) -> str:
        return "std_msgs/msg/Float32"

    def ros_msg_type(self):
        from std_msgs.msg import Float32 as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return float32_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import Float32 as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return float32_to_ros(bus)
