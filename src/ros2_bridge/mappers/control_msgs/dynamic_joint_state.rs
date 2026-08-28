//! Typed mapper for `control_msgs/msg/DynamicJointState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn dynamic_joint_state_to_bus(msg: ros_env::control_msgs::msg::DynamicJointState) -> crate::control_msgs::msg::v1::DynamicJointState {
    crate::control_msgs::msg::v1::DynamicJointState {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        joint_names: crate::ros2_bridge::mappers::convert::string_seq(msg.joint_names),
        interface_values: msg.interface_values.into_iter().map(crate::ros2_bridge::mappers::control_msgs::interface_value::interface_value_to_bus).collect(),
    }
}

pub(crate) fn dynamic_joint_state_to_ros(bus: crate::control_msgs::msg::v1::DynamicJointState) -> ros_env::control_msgs::msg::DynamicJointState {
    ros_env::control_msgs::msg::DynamicJointState {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        joint_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.joint_names),
        interface_values: bus.interface_values.into_iter().map(crate::ros2_bridge::mappers::control_msgs::interface_value::interface_value_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsDynamicJointStateMapper;

impl TypedTopicMapper for ControlMsgsDynamicJointStateMapper {
    type Ros = ros_env::control_msgs::msg::DynamicJointState;
    type Bus = crate::control_msgs::msg::v1::DynamicJointState;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/DynamicJointState"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(dynamic_joint_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(dynamic_joint_state_to_ros(msg))
    }
}
