"""Generated mapper for `control_msgs/msg/DynamicInterfaceGroupValues`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.control_msgs.interface_value import interface_value_to_bus, interface_value_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import DynamicInterfaceGroupValues as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def dynamic_interface_group_values_to_bus(msg):
    BusMsg = _bus_cls()
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
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import DynamicInterfaceGroupValues as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return dynamic_interface_group_values_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return dynamic_interface_group_values_to_ros(bus)
