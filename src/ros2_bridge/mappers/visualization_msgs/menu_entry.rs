//! Typed mapper for `visualization_msgs/msg/MenuEntry`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn menu_entry_to_bus(
    msg: ros_env::visualization_msgs::msg::MenuEntry,
) -> crate::visualization_msgs::msg::v1::MenuEntry {
    crate::visualization_msgs::msg::v1::MenuEntry {
        id: msg.id.into(),
        parent_id: msg.parent_id.into(),
        title: crate::ros2_bridge::mappers::convert::from_ros_string(msg.title),
        command: crate::ros2_bridge::mappers::convert::from_ros_string(msg.command),
        command_type: msg.command_type.into(),
    }
}

pub(crate) fn menu_entry_to_ros(
    bus: crate::visualization_msgs::msg::v1::MenuEntry,
) -> ros_env::visualization_msgs::msg::MenuEntry {
    ros_env::visualization_msgs::msg::MenuEntry {
        id: bus.id as _,
        parent_id: bus.parent_id as _,
        title: crate::ros2_bridge::mappers::convert::to_ros_string(bus.title),
        command: crate::ros2_bridge::mappers::convert::to_ros_string(bus.command),
        command_type: bus.command_type as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsMenuEntryMapper;

impl TypedTopicMapper for VisualizationMsgsMenuEntryMapper {
    type Ros = ros_env::visualization_msgs::msg::MenuEntry;
    type Bus = crate::visualization_msgs::msg::v1::MenuEntry;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(menu_entry_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(menu_entry_to_ros(msg))
    }
}
