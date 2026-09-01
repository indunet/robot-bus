"""Generated mapper for `control_msgs/msg/GripperCommand`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert


def gripper_command_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import GripperCommand as BusMsg

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
    def type_name(self) -> str:
        return "control_msgs/msg/GripperCommand"

    def ros_msg_type(self):
        from control_msgs.msg import GripperCommand as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return gripper_command_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import GripperCommand as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return gripper_command_to_ros(bus)
