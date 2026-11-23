//! Typed mapper for `visualization_msgs/msg/InteractiveMarker`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn interactive_marker_to_bus(
    msg: ros_env::visualization_msgs::msg::InteractiveMarker,
) -> robot_bus::visualization_msgs::msg::v1::InteractiveMarker {
    robot_bus::visualization_msgs::msg::v1::InteractiveMarker {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        pose: Some(crate::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        name: crate::mappers::convert::from_ros_string(msg.name),
        description: crate::mappers::convert::from_ros_string(msg.description),
        scale: msg.scale,
        menu_entries: msg.menu_entries.into_iter().map(crate::mappers::visualization_msgs::menu_entry::menu_entry_to_bus).collect(),
        controls: msg.controls.into_iter().map(crate::mappers::visualization_msgs::interactive_marker_control::interactive_marker_control_to_bus).collect(),
    }
}

pub(crate) fn interactive_marker_to_ros(
    bus: robot_bus::visualization_msgs::msg::v1::InteractiveMarker,
) -> ros_env::visualization_msgs::msg::InteractiveMarker {
    ros_env::visualization_msgs::msg::InteractiveMarker {
        header: crate::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        pose: crate::mappers::geometry_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        name: crate::mappers::convert::to_ros_string(bus.name),
        description: crate::mappers::convert::to_ros_string(bus.description),
        scale: bus.scale,
        menu_entries: bus.menu_entries.into_iter().map(crate::mappers::visualization_msgs::menu_entry::menu_entry_to_ros).collect(),
        controls: bus.controls.into_iter().map(crate::mappers::visualization_msgs::interactive_marker_control::interactive_marker_control_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsInteractiveMarkerMapper;

impl TypedTopicMapper for VisualizationMsgsInteractiveMarkerMapper {
    type Ros = ros_env::visualization_msgs::msg::InteractiveMarker;
    type Bus = robot_bus::visualization_msgs::msg::v1::InteractiveMarker;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(interactive_marker_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(interactive_marker_to_ros(msg))
    }
}
