//! Typed mapper for `foxglove_msgs/msg/CylinderPrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn cylinder_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::CylinderPrimitive) -> crate::foxglove_msgs::msg::v1::CylinderPrimitive {
    crate::foxglove_msgs::msg::v1::CylinderPrimitive {
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        size: Some(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus(msg.size)),
        bottom_scale: msg.bottom_scale,
        top_scale: msg.top_scale,
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
    }
}

pub(crate) fn cylinder_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::CylinderPrimitive) -> ros_env::foxglove_msgs::msg::CylinderPrimitive {
    ros_env::foxglove_msgs::msg::CylinderPrimitive {
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        size: crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros(bus.size.unwrap_or_default()),
        bottom_scale: bus.bottom_scale,
        top_scale: bus.top_scale,
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsCylinderPrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsCylinderPrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::CylinderPrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::CylinderPrimitive;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CylinderPrimitive"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(cylinder_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(cylinder_primitive_to_ros(msg))
    }
}
