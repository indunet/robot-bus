//! Typed mapper for `nav_msgs/msg/Goals`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn goals_to_bus(msg: ros_env::nav_msgs::msg::Goals) -> robot_bus::nav_msgs::msg::v1::Goals {
    robot_bus::nav_msgs::msg::v1::Goals {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        goals: msg
            .goals
            .into_iter()
            .map(crate::mappers::geometry_msgs::pose_stamped::pose_stamped_to_bus)
            .collect(),
    }
}

pub(crate) fn goals_to_ros(bus: robot_bus::nav_msgs::msg::v1::Goals) -> ros_env::nav_msgs::msg::Goals {
    ros_env::nav_msgs::msg::Goals {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        goals: bus
            .goals
            .into_iter()
            .map(crate::mappers::geometry_msgs::pose_stamped::pose_stamped_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsGoalsMapper;

impl TypedTopicMapper for NavMsgsGoalsMapper {
    type Ros = ros_env::nav_msgs::msg::Goals;
    type Bus = robot_bus::nav_msgs::msg::v1::Goals;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(goals_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(goals_to_ros(msg))
    }
}
