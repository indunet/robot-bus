//! Typed mapper for `foxglove_msgs/msg/CubePrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn cube_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::CubePrimitive) -> crate::foxglove_msgs::msg::v1::CubePrimitive {
    crate::foxglove_msgs::msg::v1::CubePrimitive {
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        size: Some(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus(msg.size)),
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
    }
}

pub(crate) fn cube_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::CubePrimitive) -> ros_env::foxglove_msgs::msg::CubePrimitive {
    ros_env::foxglove_msgs::msg::CubePrimitive {
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        size: crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros(bus.size.unwrap_or_default()),
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsCubePrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsCubePrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::CubePrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::CubePrimitive;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CubePrimitive"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(cube_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(cube_primitive_to_ros(msg))
    }
}
