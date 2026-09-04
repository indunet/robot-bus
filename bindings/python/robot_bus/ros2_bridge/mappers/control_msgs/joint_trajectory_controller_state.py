"""Generated mapper for `control_msgs/msg/JointTrajectoryControllerState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros
from robot_bus.ros2_bridge.mappers.trajectory_msgs.joint_trajectory_point import joint_trajectory_point_to_bus, joint_trajectory_point_to_ros
from robot_bus.ros2_bridge.mappers.trajectory_msgs.multi_dof_joint_trajectory_point import multi_dof_joint_trajectory_point_to_bus, multi_dof_joint_trajectory_point_to_ros

_BusMsg = None


def _bus_cls():
    global _BusMsg
    if _BusMsg is None:
        from robot_bus.control_msgs.msg.v1 import JointTrajectoryControllerState as BusMsg

        _BusMsg = BusMsg
    return _BusMsg


def joint_trajectory_controller_state_to_bus(msg):
    BusMsg = _bus_cls()
    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.joint_names.extend([str(x) for x in msg.joint_names])
    bus.reference.CopyFrom(joint_trajectory_point_to_bus(msg.reference))
    bus.feedback.CopyFrom(joint_trajectory_point_to_bus(msg.feedback))
    bus.error.CopyFrom(joint_trajectory_point_to_bus(msg.error))
    bus.output.CopyFrom(joint_trajectory_point_to_bus(msg.output))
    bus.multi_dof_joint_names.extend([str(x) for x in msg.multi_dof_joint_names])
    bus.multi_dof_reference.CopyFrom(multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_reference))
    bus.multi_dof_feedback.CopyFrom(multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_feedback))
    bus.multi_dof_error.CopyFrom(multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_error))
    bus.multi_dof_output.CopyFrom(multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_output))
    return bus


def joint_trajectory_controller_state_to_ros(bus):
    from control_msgs.msg import JointTrajectoryControllerState as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.joint_names = [str(x) for x in bus.joint_names]
    out.reference = joint_trajectory_point_to_ros(bus.reference)
    out.feedback = joint_trajectory_point_to_ros(bus.feedback)
    out.error = joint_trajectory_point_to_ros(bus.error)
    out.output = joint_trajectory_point_to_ros(bus.output)
    out.multi_dof_joint_names = [str(x) for x in bus.multi_dof_joint_names]
    out.multi_dof_reference = multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_reference)
    out.multi_dof_feedback = multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_feedback)
    out.multi_dof_error = multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_error)
    out.multi_dof_output = multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_output)
    return out


class ControlMsgsJointTrajectoryControllerStateMapper:
    _ros_type = None

    def ros_msg_type(self):
        cls = type(self)
        if cls._ros_type is None:
            from control_msgs.msg import JointTrajectoryControllerState as RosMsg

            cls._ros_type = RosMsg
        return cls._ros_type

    def ros_to_bus(self, msg) -> bytes:
        return joint_trajectory_controller_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        BusMsg = _bus_cls()
        bus = BusMsg()
        bus.ParseFromString(payload)
        return joint_trajectory_controller_state_to_ros(bus)
