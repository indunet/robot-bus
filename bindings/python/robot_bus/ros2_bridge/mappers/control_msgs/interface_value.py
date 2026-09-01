"""Generated mapper for `control_msgs/msg/InterfaceValue`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def interface_value_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import InterfaceValue as BusMsg

    bus = BusMsg()
    bus.interface_names.extend([str(x) for x in msg.interface_names])
    bus.values.extend(list(msg.values))
    return bus


def interface_value_to_ros(bus):
    from control_msgs.msg import InterfaceValue as RosMsg

    out = RosMsg()
    out.interface_names = [str(x) for x in bus.interface_names]
    out.values = list(bus.values)
    return out


class ControlMsgsInterfaceValueMapper:
    def type_name(self) -> str:
        return "control_msgs/msg/InterfaceValue"

    def ros_msg_type(self):
        from control_msgs.msg import InterfaceValue as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return interface_value_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import InterfaceValue as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return interface_value_to_ros(bus)
