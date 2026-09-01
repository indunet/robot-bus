"""Generated mapper for `control_msgs/msg/AdmittanceControllerState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform_stamped import transform_stamped_to_bus, transform_stamped_to_ros
from robot_bus.ros2_bridge.mappers.std_msgs.float64_multi_array import float64_multi_array_to_bus, float64_multi_array_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist_stamped import twist_stamped_to_bus, twist_stamped_to_ros
from robot_bus.ros2_bridge.mappers.geometry_msgs.wrench_stamped import wrench_stamped_to_bus, wrench_stamped_to_ros
from robot_bus.ros2_bridge.mappers.sensor_msgs.joint_state import joint_state_to_bus, joint_state_to_ros

def admittance_controller_state_to_bus(msg):
    from robot_bus.control_msgs.msg.v1 import AdmittanceControllerState as BusMsg

    bus = BusMsg()
    bus.ref_trans_base_fts.CopyFrom(transform_stamped_to_bus(msg.ref_trans_base_fts))
    bus.selected_axes.CopyFrom(float64_multi_array_to_bus(msg.selected_axes))
    bus.ft_sensor_frame.CopyFrom(transform_stamped_to_bus(msg.ft_sensor_frame))
    bus.admittance_position.CopyFrom(transform_stamped_to_bus(msg.admittance_position))
    bus.admittance_acceleration.CopyFrom(twist_stamped_to_bus(msg.admittance_acceleration))
    bus.admittance_velocity.CopyFrom(twist_stamped_to_bus(msg.admittance_velocity))
    bus.wrench_base.CopyFrom(wrench_stamped_to_bus(msg.wrench_base))
    bus.robot_ref_trans_base_fts.CopyFrom(transform_stamped_to_bus(msg.robot_ref_trans_base_fts))
    bus.joint_names.extend([str(x) for x in msg.joint_names])
    bus.joint_state.CopyFrom(joint_state_to_bus(msg.joint_state))
    return bus


def admittance_controller_state_to_ros(bus):
    from control_msgs.msg import AdmittanceControllerState as RosMsg

    out = RosMsg()
    out.ref_trans_base_fts = transform_stamped_to_ros(bus.ref_trans_base_fts)
    out.selected_axes = float64_multi_array_to_ros(bus.selected_axes)
    out.ft_sensor_frame = transform_stamped_to_ros(bus.ft_sensor_frame)
    out.admittance_position = transform_stamped_to_ros(bus.admittance_position)
    out.admittance_acceleration = twist_stamped_to_ros(bus.admittance_acceleration)
    out.admittance_velocity = twist_stamped_to_ros(bus.admittance_velocity)
    out.wrench_base = wrench_stamped_to_ros(bus.wrench_base)
    out.robot_ref_trans_base_fts = transform_stamped_to_ros(bus.robot_ref_trans_base_fts)
    out.joint_names = [str(x) for x in bus.joint_names]
    out.joint_state = joint_state_to_ros(bus.joint_state)
    return out


class ControlMsgsAdmittanceControllerStateMapper:
    def ros_msg_type(self):
        from control_msgs.msg import AdmittanceControllerState as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return admittance_controller_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.control_msgs.msg.v1 import AdmittanceControllerState as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return admittance_controller_state_to_ros(bus)
