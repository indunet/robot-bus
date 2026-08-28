//! Typed mapper for `foxglove_msgs/msg/SpherePrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn sphere_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::SpherePrimitive) -> crate::foxglove_msgs::msg::v1::SpherePrimitive {
    crate::foxglove_msgs::msg::v1::SpherePrimitive {
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        size: Some(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus(msg.size)),
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
    }
}

pub(crate) fn sphere_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::SpherePrimitive) -> ros_env::foxglove_msgs::msg::SpherePrimitive {
    ros_env::foxglove_msgs::msg::SpherePrimitive {
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        size: crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros(bus.size.unwrap_or_default()),
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsSpherePrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsSpherePrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::SpherePrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::SpherePrimitive;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/SpherePrimitive"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(sphere_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(sphere_primitive_to_ros(msg))
    }
}
