//! Typed mapper for `control_msgs/msg/GripperCommand`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn gripper_command_to_bus(msg: ros_env::control_msgs::msg::GripperCommand) -> crate::control_msgs::msg::v1::GripperCommand {
    crate::control_msgs::msg::v1::GripperCommand {
        position: msg.position,
        max_effort: msg.max_effort,
    }
}

pub(crate) fn gripper_command_to_ros(bus: crate::control_msgs::msg::v1::GripperCommand) -> ros_env::control_msgs::msg::GripperCommand {
    ros_env::control_msgs::msg::GripperCommand {
        position: bus.position,
        max_effort: bus.max_effort,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsGripperCommandMapper;

impl TypedTopicMapper for ControlMsgsGripperCommandMapper {
    type Ros = ros_env::control_msgs::msg::GripperCommand;
    type Bus = crate::control_msgs::msg::v1::GripperCommand;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(gripper_command_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(gripper_command_to_ros(msg))
    }
}
