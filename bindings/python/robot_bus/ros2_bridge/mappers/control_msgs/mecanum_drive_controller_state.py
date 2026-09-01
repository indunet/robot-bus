"""Generated mapper for `control_msgs/msg/MecanumDriveControllerState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import twist_to_bus, twist_to_ros

def mecanum_drive_controller_state_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import MecanumDriveControllerState as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.front_left_wheel_velocity = msg.front_left_wheel_velocity
    bus.front_right_wheel_velocity = msg.front_right_wheel_velocity
    bus.back_left_wheel_velocity = msg.back_left_wheel_velocity
    bus.back_right_wheel_velocity = msg.back_right_wheel_velocity
    bus.reference_velocity.CopyFrom(twist_to_bus(msg.reference_velocity))
    return bus


def mecanum_drive_controller_state_to_ros(bus):
    from control_msgs.msg import MecanumDriveControllerState as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.front_left_wheel_velocity = bus.front_left_wheel_velocity
    out.front_right_wheel_velocity = bus.front_right_wheel_velocity
    out.back_left_wheel_velocity = bus.back_left_wheel_velocity
    out.back_right_wheel_velocity = bus.back_right_wheel_velocity
    out.reference_velocity = twist_to_ros(bus.reference_velocity)
    return out


class ControlMsgsMecanumDriveControllerStateMapper:
    def type_name(self) -> str:
        return "control_msgs/msg/MecanumDriveControllerState"

    def ros_msg_type(self):
        from control_msgs.msg import MecanumDriveControllerState as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return mecanum_drive_controller_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import MecanumDriveControllerState as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return mecanum_drive_controller_state_to_ros(bus)
