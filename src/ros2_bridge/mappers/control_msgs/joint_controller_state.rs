//! Typed mapper for `control_msgs/msg/JointControllerState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_controller_state_to_bus(msg: ros_env::control_msgs::msg::JointControllerState) -> crate::control_msgs::msg::v1::JointControllerState {
    crate::control_msgs::msg::v1::JointControllerState {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        set_point: msg.set_point,
        process_value: msg.process_value,
        process_value_dot: msg.process_value_dot,
        error: msg.error,
        time_step: msg.time_step,
        command: msg.command,
        p: msg.p,
        i: msg.i,
        d: msg.d,
        i_clamp: msg.i_clamp,
        antiwindup: msg.antiwindup,
    }
}

pub(crate) fn joint_controller_state_to_ros(bus: crate::control_msgs::msg::v1::JointControllerState) -> ros_env::control_msgs::msg::JointControllerState {
    ros_env::control_msgs::msg::JointControllerState {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        set_point: bus.set_point,
        process_value: bus.process_value,
        process_value_dot: bus.process_value_dot,
        error: bus.error,
        time_step: bus.time_step,
        command: bus.command,
        p: bus.p,
        i: bus.i,
        d: bus.d,
        i_clamp: bus.i_clamp,
        antiwindup: bus.antiwindup,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsJointControllerStateMapper;

impl TypedTopicMapper for ControlMsgsJointControllerStateMapper {
    type Ros = ros_env::control_msgs::msg::JointControllerState;
    type Bus = crate::control_msgs::msg::v1::JointControllerState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_controller_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_controller_state_to_ros(msg))
    }
}
