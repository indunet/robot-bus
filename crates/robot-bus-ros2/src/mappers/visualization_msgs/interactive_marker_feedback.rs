//! Typed mapper for `visualization_msgs/msg/InteractiveMarkerFeedback`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn interactive_marker_feedback_to_bus(
    msg: ros_env::visualization_msgs::msg::InteractiveMarkerFeedback,
) -> robot_bus::visualization_msgs::msg::v1::InteractiveMarkerFeedback {
    robot_bus::visualization_msgs::msg::v1::InteractiveMarkerFeedback {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        client_id: crate::mappers::convert::from_ros_string(msg.client_id),
        marker_name: crate::mappers::convert::from_ros_string(msg.marker_name),
        control_name: crate::mappers::convert::from_ros_string(msg.control_name),
        event_type: msg.event_type.into(),
        pose: Some(crate::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        menu_entry_id: msg.menu_entry_id.into(),
        mouse_point: Some(
            crate::mappers::geometry_msgs::point::point_to_bus(msg.mouse_point),
        ),
        mouse_point_valid: msg.mouse_point_valid,
    }
}

pub(crate) fn interactive_marker_feedback_to_ros(
    bus: robot_bus::visualization_msgs::msg::v1::InteractiveMarkerFeedback,
) -> ros_env::visualization_msgs::msg::InteractiveMarkerFeedback {
    ros_env::visualization_msgs::msg::InteractiveMarkerFeedback {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        client_id: crate::mappers::convert::to_ros_string(bus.client_id),
        marker_name: crate::mappers::convert::to_ros_string(bus.marker_name),
        control_name: crate::mappers::convert::to_ros_string(bus.control_name),
        event_type: bus.event_type as _,
        pose: crate::mappers::geometry_msgs::pose::pose_to_ros(
            bus.pose.unwrap_or_default(),
        ),
        menu_entry_id: bus.menu_entry_id as _,
        mouse_point: crate::mappers::geometry_msgs::point::point_to_ros(
            bus.mouse_point.unwrap_or_default(),
        ),
        mouse_point_valid: bus.mouse_point_valid,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsInteractiveMarkerFeedbackMapper;

impl TypedTopicMapper for VisualizationMsgsInteractiveMarkerFeedbackMapper {
    type Ros = ros_env::visualization_msgs::msg::InteractiveMarkerFeedback;
    type Bus = robot_bus::visualization_msgs::msg::v1::InteractiveMarkerFeedback;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(interactive_marker_feedback_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(interactive_marker_feedback_to_ros(msg))
    }
}
