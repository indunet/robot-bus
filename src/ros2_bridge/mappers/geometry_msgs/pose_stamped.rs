//! Typed mapper for `geometry_msgs/msg/PoseStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn pose_stamped_to_bus(msg: ros_env::geometry_msgs::msg::PoseStamped) -> crate::geometry_msgs::msg::v1::PoseStamped {
    crate::geometry_msgs::msg::v1::PoseStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        pose: Some(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
    }
}

pub(crate) fn pose_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::PoseStamped) -> ros_env::geometry_msgs::msg::PoseStamped {
    ros_env::geometry_msgs::msg::PoseStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        pose: crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPoseStampedMapper;

impl TypedTopicMapper for GeometryMsgsPoseStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::PoseStamped;
    type Bus = crate::geometry_msgs::msg::v1::PoseStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PoseStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(pose_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(pose_stamped_to_ros(msg))
    }
}
