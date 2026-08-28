//! Typed mapper for `nav2_msgs/msg/MissedWaypoint`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn missed_waypoint_to_bus(msg: ros_env::nav2_msgs::msg::MissedWaypoint) -> crate::nav2_msgs::msg::v1::MissedWaypoint {
    crate::nav2_msgs::msg::v1::MissedWaypoint {
        index: msg.index,
        goal: Some(crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_bus(msg.goal)),
        error_code: msg.error_code,
    }
}

pub(crate) fn missed_waypoint_to_ros(bus: crate::nav2_msgs::msg::v1::MissedWaypoint) -> ros_env::nav2_msgs::msg::MissedWaypoint {
    ros_env::nav2_msgs::msg::MissedWaypoint {
        index: bus.index,
        goal: crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_ros(bus.goal.unwrap_or_default()),
        error_code: bus.error_code,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsMissedWaypointMapper;

impl TypedTopicMapper for Nav2MsgsMissedWaypointMapper {
    type Ros = ros_env::nav2_msgs::msg::MissedWaypoint;
    type Bus = crate::nav2_msgs::msg::v1::MissedWaypoint;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/MissedWaypoint"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(missed_waypoint_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(missed_waypoint_to_ros(msg))
    }
}
