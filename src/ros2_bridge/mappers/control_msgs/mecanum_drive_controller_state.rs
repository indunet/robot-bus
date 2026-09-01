//! Typed mapper for `control_msgs/msg/MecanumDriveControllerState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn mecanum_drive_controller_state_to_bus(msg: ros_env::control_msgs::msg::MecanumDriveControllerState) -> crate::control_msgs::msg::v1::MecanumDriveControllerState {
    crate::control_msgs::msg::v1::MecanumDriveControllerState {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        front_left_wheel_velocity: msg.front_left_wheel_velocity,
        front_right_wheel_velocity: msg.front_right_wheel_velocity,
        back_left_wheel_velocity: msg.back_left_wheel_velocity,
        back_right_wheel_velocity: msg.back_right_wheel_velocity,
        reference_velocity: Some(crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_bus(msg.reference_velocity)),
    }
}

pub(crate) fn mecanum_drive_controller_state_to_ros(bus: crate::control_msgs::msg::v1::MecanumDriveControllerState) -> ros_env::control_msgs::msg::MecanumDriveControllerState {
    ros_env::control_msgs::msg::MecanumDriveControllerState {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        front_left_wheel_velocity: bus.front_left_wheel_velocity,
        front_right_wheel_velocity: bus.front_right_wheel_velocity,
        back_left_wheel_velocity: bus.back_left_wheel_velocity,
        back_right_wheel_velocity: bus.back_right_wheel_velocity,
        reference_velocity: crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_ros(bus.reference_velocity.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsMecanumDriveControllerStateMapper;

impl TypedTopicMapper for ControlMsgsMecanumDriveControllerStateMapper {
    type Ros = ros_env::control_msgs::msg::MecanumDriveControllerState;
    type Bus = crate::control_msgs::msg::v1::MecanumDriveControllerState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(mecanum_drive_controller_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(mecanum_drive_controller_state_to_ros(msg))
    }
}
