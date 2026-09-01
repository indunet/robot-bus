//! Typed mapper for `foxglove_msgs/msg/TriangleListPrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn triangle_list_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::TriangleListPrimitive) -> crate::foxglove_msgs::msg::v1::TriangleListPrimitive {
    crate::foxglove_msgs::msg::v1::TriangleListPrimitive {
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        points: msg.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::point3::point3_to_bus).collect(),
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
        colors: msg.colors.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus).collect(),
        indices: crate::ros2_bridge::mappers::convert::u32_seq(msg.indices),
    }
}

pub(crate) fn triangle_list_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::TriangleListPrimitive) -> ros_env::foxglove_msgs::msg::TriangleListPrimitive {
    ros_env::foxglove_msgs::msg::TriangleListPrimitive {
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        points: bus.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::point3::point3_to_ros).collect(),
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
        colors: bus.colors.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros).collect(),
        indices: crate::ros2_bridge::mappers::convert::FromU32Seq::from_u32_seq(bus.indices),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsTriangleListPrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsTriangleListPrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::TriangleListPrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::TriangleListPrimitive;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(triangle_list_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(triangle_list_primitive_to_ros(msg))
    }
}
