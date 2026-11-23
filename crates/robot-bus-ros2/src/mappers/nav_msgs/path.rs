//! Typed mapper for `nav_msgs/msg/Path`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn path_to_bus(msg: ros_env::nav_msgs::msg::Path) -> robot_bus::nav_msgs::msg::v1::Path {
    robot_bus::nav_msgs::msg::v1::Path {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        poses: msg
            .poses
            .into_iter()
            .map(crate::mappers::geometry_msgs::pose_stamped::pose_stamped_to_bus)
            .collect(),
    }
}

pub(crate) fn path_to_ros(bus: robot_bus::nav_msgs::msg::v1::Path) -> ros_env::nav_msgs::msg::Path {
    ros_env::nav_msgs::msg::Path {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        poses: bus
            .poses
            .into_iter()
            .map(crate::mappers::geometry_msgs::pose_stamped::pose_stamped_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsPathMapper;

impl TypedTopicMapper for NavMsgsPathMapper {
    type Ros = ros_env::nav_msgs::msg::Path;
    type Bus = robot_bus::nav_msgs::msg::v1::Path;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(path_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(path_to_ros(msg))
    }
}
