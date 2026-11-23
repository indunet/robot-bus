//! Typed mapper for `visualization_msgs/msg/InteractiveMarkerUpdate`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn interactive_marker_update_to_bus(
    msg: ros_env::visualization_msgs::msg::InteractiveMarkerUpdate,
) -> robot_bus::visualization_msgs::msg::v1::InteractiveMarkerUpdate {
    robot_bus::visualization_msgs::msg::v1::InteractiveMarkerUpdate {
        server_id: crate::mappers::convert::from_ros_string(msg.server_id),
        seq_num: msg.seq_num,
        r#type: msg.type_.into(),
        markers: msg.markers.into_iter().map(crate::mappers::visualization_msgs::interactive_marker::interactive_marker_to_bus).collect(),
        poses: msg.poses.into_iter().map(crate::mappers::visualization_msgs::interactive_marker_pose::interactive_marker_pose_to_bus).collect(),
        erases: crate::mappers::convert::string_seq(msg.erases),
    }
}

pub(crate) fn interactive_marker_update_to_ros(
    bus: robot_bus::visualization_msgs::msg::v1::InteractiveMarkerUpdate,
) -> ros_env::visualization_msgs::msg::InteractiveMarkerUpdate {
    ros_env::visualization_msgs::msg::InteractiveMarkerUpdate {
        server_id: crate::mappers::convert::to_ros_string(bus.server_id),
        seq_num: bus.seq_num,
        type_: bus.r#type as _,
        markers: bus.markers.into_iter().map(crate::mappers::visualization_msgs::interactive_marker::interactive_marker_to_ros).collect(),
        poses: bus.poses.into_iter().map(crate::mappers::visualization_msgs::interactive_marker_pose::interactive_marker_pose_to_ros).collect(),
        erases: crate::mappers::convert::ros_string_seq(bus.erases),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VisualizationMsgsInteractiveMarkerUpdateMapper;

impl TypedTopicMapper for VisualizationMsgsInteractiveMarkerUpdateMapper {
    type Ros = ros_env::visualization_msgs::msg::InteractiveMarkerUpdate;
    type Bus = robot_bus::visualization_msgs::msg::v1::InteractiveMarkerUpdate;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(interactive_marker_update_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(interactive_marker_update_to_ros(msg))
    }
}
