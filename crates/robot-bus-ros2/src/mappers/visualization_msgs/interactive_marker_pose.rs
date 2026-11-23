//! Typed mapper for `visualization_msgs/msg/InteractiveMarkerPose`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn interactive_marker_pose_to_bus(
    msg: ros_env::visualization_msgs::msg::InteractiveMarkerPose,
) -> robot_bus::visualization_msgs::msg::v1::InteractiveMarkerPose {
    robot_bus::visualization_msgs::msg::v1::InteractiveMarkerPose {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        pose: Some(crate::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        name: crate::mappers::convert::from_ros_string(msg.name),
    }
}

pub(crate) fn interactive_marker_pose_to_ros(
    bus: robot_bus::visualization_msgs::msg::v1::InteractiveMarkerPose,
) -> ros_env::visualization_msgs::msg::InteractiveMarkerPose {
    ros_env::visualization_msgs::msg::InteractiveMarkerPose {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        pose: crate::mappers::geometry_msgs::pose::pose_to_ros(
            bus.pose.unwrap_or_default(),
        ),
        name: crate::mappers::convert::to_ros_string(bus.name),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsInteractiveMarkerPoseMapper;

impl TypedTopicMapper for VisualizationMsgsInteractiveMarkerPoseMapper {
    type Ros = ros_env::visualization_msgs::msg::InteractiveMarkerPose;
    type Bus = robot_bus::visualization_msgs::msg::v1::InteractiveMarkerPose;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(interactive_marker_pose_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(interactive_marker_pose_to_ros(msg))
    }
}
