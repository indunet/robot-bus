//! Typed mapper for `nav_msgs/msg/Path`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn path_to_bus(msg: ros_env::nav_msgs::msg::Path) -> crate::nav_msgs::msg::v1::Path {
    crate::nav_msgs::msg::v1::Path {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        poses: msg.poses.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_bus).collect(),
    }
}

pub(crate) fn path_to_ros(bus: crate::nav_msgs::msg::v1::Path) -> ros_env::nav_msgs::msg::Path {
    ros_env::nav_msgs::msg::Path {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        poses: bus.poses.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::pose_stamped::pose_stamped_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsPathMapper;

impl TypedTopicMapper for NavMsgsPathMapper {
    type Ros = ros_env::nav_msgs::msg::Path;
    type Bus = crate::nav_msgs::msg::v1::Path;

    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/Path"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(path_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(path_to_ros(msg))
    }
}
