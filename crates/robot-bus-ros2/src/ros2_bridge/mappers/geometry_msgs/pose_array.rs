//! Typed mapper for `geometry_msgs/msg/PoseArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn pose_array_to_bus(
    msg: ros_env::geometry_msgs::msg::PoseArray,
) -> crate::geometry_msgs::msg::v1::PoseArray {
    crate::geometry_msgs::msg::v1::PoseArray {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        poses: msg
            .poses
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_bus)
            .collect(),
    }
}

pub(crate) fn pose_array_to_ros(
    bus: crate::geometry_msgs::msg::v1::PoseArray,
) -> ros_env::geometry_msgs::msg::PoseArray {
    ros_env::geometry_msgs::msg::PoseArray {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        poses: bus
            .poses
            .into_iter()
            .map(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPoseArrayMapper;

impl TypedTopicMapper for GeometryMsgsPoseArrayMapper {
    type Ros = ros_env::geometry_msgs::msg::PoseArray;
    type Bus = crate::geometry_msgs::msg::v1::PoseArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(pose_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(pose_array_to_ros(msg))
    }
}
