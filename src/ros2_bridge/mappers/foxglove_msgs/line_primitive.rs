//! Typed mapper for `foxglove_msgs/msg/LinePrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn line_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::LinePrimitive) -> crate::foxglove_msgs::msg::v1::LinePrimitive {
    crate::foxglove_msgs::msg::v1::LinePrimitive {
        r#type: msg.type_ as i32,
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        thickness: msg.thickness,
        scale_invariant: msg.scale_invariant,
        points: msg.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::point3::point3_to_bus).collect(),
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
        colors: msg.colors.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus).collect(),
        indices: crate::ros2_bridge::mappers::convert::u32_seq(msg.indices),
    }
}

pub(crate) fn line_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::LinePrimitive) -> ros_env::foxglove_msgs::msg::LinePrimitive {
    ros_env::foxglove_msgs::msg::LinePrimitive {
        type_: bus.r#type as i32,
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        thickness: bus.thickness,
        scale_invariant: bus.scale_invariant,
        points: bus.points.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::point3::point3_to_ros).collect(),
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
        colors: bus.colors.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros).collect(),
        indices: crate::ros2_bridge::mappers::convert::FromU32Seq::from_u32_seq(bus.indices),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsLinePrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsLinePrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::LinePrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::LinePrimitive;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/LinePrimitive"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(line_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(line_primitive_to_ros(msg))
    }
}
