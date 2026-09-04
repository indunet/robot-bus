//! Typed mapper for `visualization_msgs/msg/InteractiveMarkerControl`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn interactive_marker_control_to_bus(msg: ros_env::visualization_msgs::msg::InteractiveMarkerControl) -> crate::visualization_msgs::msg::v1::InteractiveMarkerControl {
    crate::visualization_msgs::msg::v1::InteractiveMarkerControl {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        orientation: Some(crate::ros2_bridge::mappers::geometry_msgs::quaternion::quaternion_to_bus(msg.orientation)),
        orientation_mode: msg.orientation_mode.into(),
        interaction_mode: msg.interaction_mode.into(),
        always_visible: msg.always_visible,
        markers: msg.markers.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::marker::marker_to_bus).collect(),
        independent_marker_orientation: msg.independent_marker_orientation,
        description: crate::ros2_bridge::mappers::convert::from_ros_string(msg.description),
    }
}

pub(crate) fn interactive_marker_control_to_ros(bus: crate::visualization_msgs::msg::v1::InteractiveMarkerControl) -> ros_env::visualization_msgs::msg::InteractiveMarkerControl {
    ros_env::visualization_msgs::msg::InteractiveMarkerControl {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        orientation: crate::ros2_bridge::mappers::geometry_msgs::quaternion::quaternion_to_ros(bus.orientation.unwrap_or_default()),
        orientation_mode: bus.orientation_mode as _,
        interaction_mode: bus.interaction_mode as _,
        always_visible: bus.always_visible,
        markers: bus.markers.into_iter().map(crate::ros2_bridge::mappers::visualization_msgs::marker::marker_to_ros).collect(),
        independent_marker_orientation: bus.independent_marker_orientation,
        description: crate::ros2_bridge::mappers::convert::to_ros_string(bus.description),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsInteractiveMarkerControlMapper;

impl TypedTopicMapper for VisualizationMsgsInteractiveMarkerControlMapper {
    type Ros = ros_env::visualization_msgs::msg::InteractiveMarkerControl;
    type Bus = crate::visualization_msgs::msg::v1::InteractiveMarkerControl;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(interactive_marker_control_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(interactive_marker_control_to_ros(msg))
    }
}
