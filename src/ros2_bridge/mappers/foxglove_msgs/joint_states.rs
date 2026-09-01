//! Typed mapper for `foxglove_msgs/msg/JointStates`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_states_to_bus(msg: ros_env::foxglove_msgs::msg::JointStates) -> crate::foxglove_msgs::msg::v1::JointStates {
    crate::foxglove_msgs::msg::v1::JointStates {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        joints: msg.joints.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::joint_state::joint_state_to_bus).collect(),
    }
}

pub(crate) fn joint_states_to_ros(bus: crate::foxglove_msgs::msg::v1::JointStates) -> ros_env::foxglove_msgs::msg::JointStates {
    ros_env::foxglove_msgs::msg::JointStates {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        joints: bus.joints.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::joint_state::joint_state_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsJointStatesMapper;

impl TypedTopicMapper for FoxgloveMsgsJointStatesMapper {
    type Ros = ros_env::foxglove_msgs::msg::JointStates;
    type Bus = crate::foxglove_msgs::msg::v1::JointStates;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_states_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_states_to_ros(msg))
    }
}
