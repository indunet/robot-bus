//! Typed mapper for `control_msgs/msg/AdmittanceControllerState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn admittance_controller_state_to_bus(msg: ros_env::control_msgs::msg::AdmittanceControllerState) -> crate::control_msgs::msg::v1::AdmittanceControllerState {
    crate::control_msgs::msg::v1::AdmittanceControllerState {
        ref_trans_base_fts: Some(crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_bus(msg.ref_trans_base_fts)),
        selected_axes: Some(crate::ros2_bridge::mappers::std_msgs::float64_multi_array::float64_multi_array_to_bus(msg.selected_axes)),
        ft_sensor_frame: Some(crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_bus(msg.ft_sensor_frame)),
        admittance_position: Some(crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_bus(msg.admittance_position)),
        admittance_acceleration: Some(crate::ros2_bridge::mappers::geometry_msgs::twist_stamped::twist_stamped_to_bus(msg.admittance_acceleration)),
        admittance_velocity: Some(crate::ros2_bridge::mappers::geometry_msgs::twist_stamped::twist_stamped_to_bus(msg.admittance_velocity)),
        wrench_base: Some(crate::ros2_bridge::mappers::geometry_msgs::wrench_stamped::wrench_stamped_to_bus(msg.wrench_base)),
        robot_ref_trans_base_fts: Some(crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_bus(msg.robot_ref_trans_base_fts)),
        joint_names: crate::ros2_bridge::mappers::convert::string_seq(msg.joint_names),
        joint_state: Some(crate::ros2_bridge::mappers::sensor_msgs::joint_state::joint_state_to_bus(msg.joint_state)),
    }
}

pub(crate) fn admittance_controller_state_to_ros(bus: crate::control_msgs::msg::v1::AdmittanceControllerState) -> ros_env::control_msgs::msg::AdmittanceControllerState {
    ros_env::control_msgs::msg::AdmittanceControllerState {
        ref_trans_base_fts: crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_ros(bus.ref_trans_base_fts.unwrap_or_default()),
        selected_axes: crate::ros2_bridge::mappers::std_msgs::float64_multi_array::float64_multi_array_to_ros(bus.selected_axes.unwrap_or_default()),
        ft_sensor_frame: crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_ros(bus.ft_sensor_frame.unwrap_or_default()),
        admittance_position: crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_ros(bus.admittance_position.unwrap_or_default()),
        admittance_acceleration: crate::ros2_bridge::mappers::geometry_msgs::twist_stamped::twist_stamped_to_ros(bus.admittance_acceleration.unwrap_or_default()),
        admittance_velocity: crate::ros2_bridge::mappers::geometry_msgs::twist_stamped::twist_stamped_to_ros(bus.admittance_velocity.unwrap_or_default()),
        wrench_base: crate::ros2_bridge::mappers::geometry_msgs::wrench_stamped::wrench_stamped_to_ros(bus.wrench_base.unwrap_or_default()),
        robot_ref_trans_base_fts: crate::ros2_bridge::mappers::geometry_msgs::transform_stamped::transform_stamped_to_ros(bus.robot_ref_trans_base_fts.unwrap_or_default()),
        joint_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.joint_names),
        joint_state: crate::ros2_bridge::mappers::sensor_msgs::joint_state::joint_state_to_ros(bus.joint_state.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsAdmittanceControllerStateMapper;

impl TypedTopicMapper for ControlMsgsAdmittanceControllerStateMapper {
    type Ros = ros_env::control_msgs::msg::AdmittanceControllerState;
    type Bus = crate::control_msgs::msg::v1::AdmittanceControllerState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(admittance_controller_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(admittance_controller_state_to_ros(msg))
    }
}
