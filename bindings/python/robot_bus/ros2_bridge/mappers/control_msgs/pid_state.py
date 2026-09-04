"""Generated mapper for `control_msgs/msg/PidState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.builtin_interfaces.duration import duration_to_bus, duration_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import PidState as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def pid_state_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.timestep.CopyFrom(duration_to_bus(msg.timestep))
    bus.error = msg.error
    bus.error_dot = msg.error_dot
    bus.p_error = msg.p_error
    bus.i_error = msg.i_error
    bus.d_error = msg.d_error
    bus.p_term = msg.p_term
    bus.i_term = msg.i_term
    bus.d_term = msg.d_term
    bus.i_max = msg.i_max
    bus.i_min = msg.i_min
    bus.output = msg.output
    return bus


def pid_state_to_ros(bus):
    from control_msgs.msg import PidState as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.timestep = duration_to_ros(bus.timestep)
    out.error = bus.error
    out.error_dot = bus.error_dot
    out.p_error = bus.p_error
    out.i_error = bus.i_error
    out.d_error = bus.d_error
    out.p_term = bus.p_term
    out.i_term = bus.i_term
    out.d_term = bus.d_term
    out.i_max = bus.i_max
    out.i_min = bus.i_min
    out.output = bus.output
    return out


class ControlMsgsPidStateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import PidState as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return pid_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return pid_state_to_ros(bus)
