"""Generated mapper for `control_msgs/msg/DynamicInterfaceGroupValues`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.control_msgs.interface_value import interface_value_to_bus, interface_value_to_ros

def dynamic_interface_group_values_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import DynamicInterfaceGroupValues as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.interface_groups.extend([str(x) for x in msg.interface_groups])
    bus.interface_values.extend([interface_value_to_bus(x) for x in msg.interface_values])
    return bus


def dynamic_interface_group_values_to_ros(bus):
    from control_msgs.msg import DynamicInterfaceGroupValues as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.interface_groups = [str(x) for x in bus.interface_groups]
    out.interface_values = [interface_value_to_ros(x) for x in bus.interface_values]
    return out


class ControlMsgsDynamicInterfaceGroupValuesMapper:
    def type_name(self) -> str:
        return "control_msgs/msg/DynamicInterfaceGroupValues"

    def ros_msg_type(self):
        from control_msgs.msg import DynamicInterfaceGroupValues as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return dynamic_interface_group_values_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import DynamicInterfaceGroupValues as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return dynamic_interface_group_values_to_ros(bus)
