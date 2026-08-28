//! Typed mapper for `control_msgs/msg/SteeringControllerStatus`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn steering_controller_status_to_bus(msg: ros_env::control_msgs::msg::SteeringControllerStatus) -> crate::control_msgs::msg::v1::SteeringControllerStatus {
    crate::control_msgs::msg::v1::SteeringControllerStatus {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        traction_wheels_position: crate::ros2_bridge::mappers::convert::f64_seq(msg.traction_wheels_position),
        traction_wheels_velocity: crate::ros2_bridge::mappers::convert::f64_seq(msg.traction_wheels_velocity),
        steer_positions: crate::ros2_bridge::mappers::convert::f64_seq(msg.steer_positions),
        linear_velocity_command: crate::ros2_bridge::mappers::convert::f64_seq(msg.linear_velocity_command),
        steering_angle_command: crate::ros2_bridge::mappers::convert::f64_seq(msg.steering_angle_command),
    }
}

pub(crate) fn steering_controller_status_to_ros(bus: crate::control_msgs::msg::v1::SteeringControllerStatus) -> ros_env::control_msgs::msg::SteeringControllerStatus {
    ros_env::control_msgs::msg::SteeringControllerStatus {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        traction_wheels_position: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.traction_wheels_position),
        traction_wheels_velocity: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.traction_wheels_velocity),
        steer_positions: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.steer_positions),
        linear_velocity_command: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.linear_velocity_command),
        steering_angle_command: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.steering_angle_command),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsSteeringControllerStatusMapper;

impl TypedTopicMapper for ControlMsgsSteeringControllerStatusMapper {
    type Ros = ros_env::control_msgs::msg::SteeringControllerStatus;
    type Bus = crate::control_msgs::msg::v1::SteeringControllerStatus;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/SteeringControllerStatus"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(steering_controller_status_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(steering_controller_status_to_ros(msg))
    }
}
