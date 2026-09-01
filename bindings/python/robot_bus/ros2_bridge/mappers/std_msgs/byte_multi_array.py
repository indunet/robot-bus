"""Generated mapper for `std_msgs/msg/ByteMultiArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.multi_array_layout import multi_array_layout_to_bus, multi_array_layout_to_ros

def byte_multi_array_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import ByteMultiArray as BusMsg

    bus = BusMsg()
    bus.layout.CopyFrom(multi_array_layout_to_bus(msg.layout))
    bus.data = bytes(msg.data)
    return bus


def byte_multi_array_to_ros(bus):
    from std_msgs.msg import ByteMultiArray as RosMsg

    out = RosMsg()
    out.layout = multi_array_layout_to_ros(bus.layout)
    out.data = bytes(bus.data)
    return out


class StdMsgsByteMultiArrayMapper:
    def type_name(self) -> str:
        return "std_msgs/msg/ByteMultiArray"

    def ros_msg_type(self):
        from std_msgs.msg import ByteMultiArray as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return byte_multi_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import ByteMultiArray as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return byte_multi_array_to_ros(bus)
