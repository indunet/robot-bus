//! Typed mapper for `control_msgs/msg/DynamicInterfaceGroupValues`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn dynamic_interface_group_values_to_bus(msg: ros_env::control_msgs::msg::DynamicInterfaceGroupValues) -> crate::control_msgs::msg::v1::DynamicInterfaceGroupValues {
    crate::control_msgs::msg::v1::DynamicInterfaceGroupValues {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        interface_groups: crate::ros2_bridge::mappers::convert::string_seq(msg.interface_groups),
        interface_values: msg.interface_values.into_iter().map(crate::ros2_bridge::mappers::control_msgs::interface_value::interface_value_to_bus).collect(),
    }
}

pub(crate) fn dynamic_interface_group_values_to_ros(bus: crate::control_msgs::msg::v1::DynamicInterfaceGroupValues) -> ros_env::control_msgs::msg::DynamicInterfaceGroupValues {
    ros_env::control_msgs::msg::DynamicInterfaceGroupValues {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        interface_groups: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.interface_groups),
        interface_values: bus.interface_values.into_iter().map(crate::ros2_bridge::mappers::control_msgs::interface_value::interface_value_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsDynamicInterfaceGroupValuesMapper;

impl TypedTopicMapper for ControlMsgsDynamicInterfaceGroupValuesMapper {
    type Ros = ros_env::control_msgs::msg::DynamicInterfaceGroupValues;
    type Bus = crate::control_msgs::msg::v1::DynamicInterfaceGroupValues;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(dynamic_interface_group_values_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(dynamic_interface_group_values_to_ros(msg))
    }
}
