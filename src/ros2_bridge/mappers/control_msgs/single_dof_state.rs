//! Typed mapper for `control_msgs/msg/SingleDOFState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn single_dof_state_to_bus(msg: ros_env::control_msgs::msg::SingleDOFState) -> crate::control_msgs::msg::v1::SingleDofState {
    crate::control_msgs::msg::v1::SingleDofState {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        reference: msg.reference,
        feedback: msg.feedback,
        feedback_dot: msg.feedback_dot,
        error: msg.error,
        error_dot: msg.error_dot,
        time_step: msg.time_step,
        output: msg.output,
    }
}

pub(crate) fn single_dof_state_to_ros(bus: crate::control_msgs::msg::v1::SingleDofState) -> ros_env::control_msgs::msg::SingleDOFState {
    ros_env::control_msgs::msg::SingleDOFState {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        reference: bus.reference,
        feedback: bus.feedback,
        feedback_dot: bus.feedback_dot,
        error: bus.error,
        error_dot: bus.error_dot,
        time_step: bus.time_step,
        output: bus.output,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsSingleDofStateMapper;

impl TypedTopicMapper for ControlMsgsSingleDofStateMapper {
    type Ros = ros_env::control_msgs::msg::SingleDOFState;
    type Bus = crate::control_msgs::msg::v1::SingleDofState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(single_dof_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(single_dof_state_to_ros(msg))
    }
}
