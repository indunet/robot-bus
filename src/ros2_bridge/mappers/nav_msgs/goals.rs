//! Typed mapper for `nav_msgs/msg/Goals`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn goals_to_bus(msg: ros_env::nav_msgs::msg::Goals) -> crate::nav_msgs::msg::v1::Goals {
    crate::nav_msgs::msg::v1::Goals {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        goals: msg.goals.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_bus).collect(),
    }
}

pub(crate) fn goals_to_ros(bus: crate::nav_msgs::msg::v1::Goals) -> ros_env::nav_msgs::msg::Goals {
    ros_env::nav_msgs::msg::Goals {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        goals: bus.goals.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsGoalsMapper;

impl TypedTopicMapper for NavMsgsGoalsMapper {
    type Ros = ros_env::nav_msgs::msg::Goals;
    type Bus = crate::nav_msgs::msg::v1::Goals;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(goals_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(goals_to_ros(msg))
    }
}
