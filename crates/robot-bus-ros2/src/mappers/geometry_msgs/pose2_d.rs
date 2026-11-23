//! Typed mapper for `geometry_msgs/msg/Pose2D`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn pose2_d_to_bus(
    msg: ros_env::geometry_msgs::msg::Pose2D,
) -> robot_bus::geometry_msgs::msg::v1::Pose2D {
    robot_bus::geometry_msgs::msg::v1::Pose2D {
        x: msg.x,
        y: msg.y,
        theta: msg.theta,
    }
}

pub(crate) fn pose2_d_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Pose2D,
) -> ros_env::geometry_msgs::msg::Pose2D {
    ros_env::geometry_msgs::msg::Pose2D {
        x: bus.x,
        y: bus.y,
        theta: bus.theta,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPose2DMapper;

impl TypedTopicMapper for GeometryMsgsPose2DMapper {
    type Ros = ros_env::geometry_msgs::msg::Pose2D;
    type Bus = robot_bus::geometry_msgs::msg::v1::Pose2D;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(pose2_d_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(pose2_d_to_ros(msg))
    }
}
