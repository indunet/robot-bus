//! Typed mapper for `geometry_msgs/msg/Pose`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn pose_to_bus(
    msg: ros_env::geometry_msgs::msg::Pose,
) -> crate::geometry_msgs::msg::v1::Pose {
    crate::geometry_msgs::msg::v1::Pose {
        position: Some(
            crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus(msg.position),
        ),
        orientation: Some(
            crate::ros2_bridge::mappers::geometry_msgs::quaternion::quaternion_to_bus(
                msg.orientation,
            ),
        ),
    }
}

pub(crate) fn pose_to_ros(
    bus: crate::geometry_msgs::msg::v1::Pose,
) -> ros_env::geometry_msgs::msg::Pose {
    ros_env::geometry_msgs::msg::Pose {
        position: crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros(
            bus.position.unwrap_or_default(),
        ),
        orientation: crate::ros2_bridge::mappers::geometry_msgs::quaternion::quaternion_to_ros(
            bus.orientation.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPoseMapper;

impl TypedTopicMapper for GeometryMsgsPoseMapper {
    type Ros = ros_env::geometry_msgs::msg::Pose;
    type Bus = crate::geometry_msgs::msg::v1::Pose;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(pose_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(pose_to_ros(msg))
    }
}
