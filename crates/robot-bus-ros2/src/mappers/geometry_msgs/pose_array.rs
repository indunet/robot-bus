//! Typed mapper for `geometry_msgs/msg/PoseArray`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn pose_array_to_bus(
    msg: ros_env::geometry_msgs::msg::PoseArray,
) -> robot_bus::geometry_msgs::msg::v1::PoseArray {
    robot_bus::geometry_msgs::msg::v1::PoseArray {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        poses: msg
            .poses
            .into_iter()
            .map(crate::mappers::geometry_msgs::pose::pose_to_bus)
            .collect(),
    }
}

pub(crate) fn pose_array_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::PoseArray,
) -> ros_env::geometry_msgs::msg::PoseArray {
    ros_env::geometry_msgs::msg::PoseArray {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        poses: bus
            .poses
            .into_iter()
            .map(crate::mappers::geometry_msgs::pose::pose_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPoseArrayMapper;

impl TypedTopicMapper for GeometryMsgsPoseArrayMapper {
    type Ros = ros_env::geometry_msgs::msg::PoseArray;
    type Bus = robot_bus::geometry_msgs::msg::v1::PoseArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(pose_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(pose_array_to_ros(msg))
    }
}
