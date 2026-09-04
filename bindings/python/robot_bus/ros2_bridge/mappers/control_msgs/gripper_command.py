"""Generated mapper for `control_msgs/msg/GripperCommand`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import GripperCommand as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def gripper_command_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.position = msg.position
    bus.max_effort = msg.max_effort
    return bus


def gripper_command_to_ros(bus):
    from control_msgs.msg import GripperCommand as RosMsg

    out = RosMsg()
    out.position = bus.position
    out.max_effort = bus.max_effort
    return out


class ControlMsgsGripperCommandMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import GripperCommand as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return gripper_command_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return gripper_command_to_ros(bus)
