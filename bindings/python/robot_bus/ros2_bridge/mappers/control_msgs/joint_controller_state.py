"""Generated mapper for `control_msgs/msg/JointControllerState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import JointControllerState as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joint_controller_state_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.set_point = msg.set_point
    bus.process_value = msg.process_value
    bus.process_value_dot = msg.process_value_dot
    bus.error = msg.error
    bus.time_step = msg.time_step
    bus.command = msg.command
    bus.p = msg.p
    bus.i = msg.i
    bus.d = msg.d
    bus.i_clamp = msg.i_clamp
    bus.antiwindup = msg.antiwindup
    return bus


def joint_controller_state_to_ros(bus):
    from control_msgs.msg import JointControllerState as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.set_point = bus.set_point
    out.process_value = bus.process_value
    out.process_value_dot = bus.process_value_dot
    out.error = bus.error
    out.time_step = bus.time_step
    out.command = bus.command
    out.p = bus.p
    out.i = bus.i
    out.d = bus.d
    out.i_clamp = bus.i_clamp
    out.antiwindup = bus.antiwindup
    return out


class ControlMsgsJointControllerStateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import JointControllerState as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joint_controller_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_controller_state_to_ros(bus)
