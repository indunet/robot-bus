"""Generated mapper for `control_msgs/msg/SteeringControllerStatus`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def steering_controller_status_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import SteeringControllerStatus as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.traction_wheels_position.extend(list(msg.traction_wheels_position))
    bus.traction_wheels_velocity.extend(list(msg.traction_wheels_velocity))
    bus.steer_positions.extend(list(msg.steer_positions))
    bus.linear_velocity_command.extend(list(msg.linear_velocity_command))
    bus.steering_angle_command.extend(list(msg.steering_angle_command))
    return bus


def steering_controller_status_to_ros(bus):
    from control_msgs.msg import SteeringControllerStatus as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.traction_wheels_position = list(bus.traction_wheels_position)
    out.traction_wheels_velocity = list(bus.traction_wheels_velocity)
    out.steer_positions = list(bus.steer_positions)
    out.linear_velocity_command = list(bus.linear_velocity_command)
    out.steering_angle_command = list(bus.steering_angle_command)
    return out


class ControlMsgsSteeringControllerStatusMapper:
    def ros_msg_type(self):
        from control_msgs.msg import SteeringControllerStatus as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return steering_controller_status_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import SteeringControllerStatus as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return steering_controller_status_to_ros(bus)
