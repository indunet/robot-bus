//! Typed mapper for `foxglove_msgs/msg/JointState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_state_to_bus(msg: ros_env::foxglove_msgs::msg::JointState) -> crate::foxglove_msgs::msg::v1::JointState {
    crate::foxglove_msgs::msg::v1::JointState {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        position: msg.position,
        velocity: msg.velocity,
        acceleration: msg.acceleration,
        effort: msg.effort,
    }
}

pub(crate) fn joint_state_to_ros(bus: crate::foxglove_msgs::msg::v1::JointState) -> ros_env::foxglove_msgs::msg::JointState {
    ros_env::foxglove_msgs::msg::JointState {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        position: bus.position,
        velocity: bus.velocity,
        acceleration: bus.acceleration,
        effort: bus.effort,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsJointStateMapper;

impl TypedTopicMapper for FoxgloveMsgsJointStateMapper {
    type Ros = ros_env::foxglove_msgs::msg::JointState;
    type Bus = crate::foxglove_msgs::msg::v1::JointState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_state_to_ros(msg))
    }
}
