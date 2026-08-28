//! Typed mapper for `nav_msgs/msg/Trajectory`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn trajectory_to_bus(msg: ros_env::nav_msgs::msg::Trajectory) -> crate::nav_msgs::msg::v1::Trajectory {
    crate::nav_msgs::msg::v1::Trajectory {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        points: msg.points.into_iter().map(crate::ros2_bridge::mappers::nav_msgs::trajectory_point::trajectory_point_to_bus).collect(),
    }
}

pub(crate) fn trajectory_to_ros(bus: crate::nav_msgs::msg::v1::Trajectory) -> ros_env::nav_msgs::msg::Trajectory {
    ros_env::nav_msgs::msg::Trajectory {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        points: bus.points.into_iter().map(crate::ros2_bridge::mappers::nav_msgs::trajectory_point::trajectory_point_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsTrajectoryMapper;

impl TypedTopicMapper for NavMsgsTrajectoryMapper {
    type Ros = ros_env::nav_msgs::msg::Trajectory;
    type Bus = crate::nav_msgs::msg::v1::Trajectory;

    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/Trajectory"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(trajectory_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(trajectory_to_ros(msg))
    }
}
