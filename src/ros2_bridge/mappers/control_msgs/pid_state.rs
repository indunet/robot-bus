//! Typed mapper for `control_msgs/msg/PidState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn pid_state_to_bus(msg: ros_env::control_msgs::msg::PidState) -> crate::control_msgs::msg::v1::PidState {
    crate::control_msgs::msg::v1::PidState {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        timestep: Some(crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_bus(msg.timestep)),
        error: msg.error,
        error_dot: msg.error_dot,
        p_error: msg.p_error,
        i_error: msg.i_error,
        d_error: msg.d_error,
        p_term: msg.p_term,
        i_term: msg.i_term,
        d_term: msg.d_term,
        i_max: msg.i_max,
        i_min: msg.i_min,
        output: msg.output,
    }
}

pub(crate) fn pid_state_to_ros(bus: crate::control_msgs::msg::v1::PidState) -> ros_env::control_msgs::msg::PidState {
    ros_env::control_msgs::msg::PidState {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        timestep: crate::ros2_bridge::mappers::builtin_interfaces::duration::duration_to_ros(bus.timestep.unwrap_or_default()),
        error: bus.error,
        error_dot: bus.error_dot,
        p_error: bus.p_error,
        i_error: bus.i_error,
        d_error: bus.d_error,
        p_term: bus.p_term,
        i_term: bus.i_term,
        d_term: bus.d_term,
        i_max: bus.i_max,
        i_min: bus.i_min,
        output: bus.output,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsPidStateMapper;

impl TypedTopicMapper for ControlMsgsPidStateMapper {
    type Ros = ros_env::control_msgs::msg::PidState;
    type Bus = crate::control_msgs::msg::v1::PidState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(pid_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(pid_state_to_ros(msg))
    }
}
