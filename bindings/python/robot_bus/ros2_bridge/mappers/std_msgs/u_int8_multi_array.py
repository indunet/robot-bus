"""Generated mapper for `std_msgs/msg/UInt8MultiArray`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.multi_array_layout import multi_array_layout_to_bus, multi_array_layout_to_ros

def u_int8_multi_array_to_bus(msg):
    from robot_bus.std_msgs.msg.v1 import UInt8MultiArray as BusMsg

    bus = BusMsg()
    bus.layout.CopyFrom(multi_array_layout_to_bus(msg.layout))
    bus.data.extend(list(msg.data))
    return bus


def u_int8_multi_array_to_ros(bus):
    from std_msgs.msg import UInt8MultiArray as RosMsg

    out = RosMsg()
    out.layout = multi_array_layout_to_ros(bus.layout)
    out.data = list(bus.data)
    return out


class StdMsgsUInt8MultiArrayMapper:
    def ros_msg_type(self):
        from std_msgs.msg import UInt8MultiArray as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return u_int8_multi_array_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.std_msgs.msg.v1 import UInt8MultiArray as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return u_int8_multi_array_to_ros(bus)
