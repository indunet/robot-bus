//! Typed mapper for `foxglove_msgs/msg/ModelPrimitive`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn model_primitive_to_bus(msg: ros_env::foxglove_msgs::msg::ModelPrimitive) -> crate::foxglove_msgs::msg::v1::ModelPrimitive {
    crate::foxglove_msgs::msg::v1::ModelPrimitive {
        pose: Some(crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_bus(msg.pose)),
        scale: Some(crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_bus(msg.scale)),
        color: Some(crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_bus(msg.color)),
        override_color: msg.override_color,
        url: crate::ros2_bridge::mappers::convert::from_ros_string(msg.url),
        media_type: crate::ros2_bridge::mappers::convert::from_ros_string(msg.media_type),
        data: crate::ros2_bridge::mappers::convert::IntoU8Vec::into_u8_vec(msg.data),
    }
}

pub(crate) fn model_primitive_to_ros(bus: crate::foxglove_msgs::msg::v1::ModelPrimitive) -> ros_env::foxglove_msgs::msg::ModelPrimitive {
    ros_env::foxglove_msgs::msg::ModelPrimitive {
        pose: crate::ros2_bridge::mappers::foxglove_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        scale: crate::ros2_bridge::mappers::foxglove_msgs::vector3::vector3_to_ros(bus.scale.unwrap_or_default()),
        color: crate::ros2_bridge::mappers::foxglove_msgs::color::color_to_ros(bus.color.unwrap_or_default()),
        override_color: bus.override_color,
        url: crate::ros2_bridge::mappers::convert::to_ros_string(bus.url),
        media_type: crate::ros2_bridge::mappers::convert::to_ros_string(bus.media_type),
        data: crate::ros2_bridge::mappers::convert::FromByteSeq::from_byte_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsModelPrimitiveMapper;

impl TypedTopicMapper for FoxgloveMsgsModelPrimitiveMapper {
    type Ros = ros_env::foxglove_msgs::msg::ModelPrimitive;
    type Bus = crate::foxglove_msgs::msg::v1::ModelPrimitive;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/ModelPrimitive"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(model_primitive_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(model_primitive_to_ros(msg))
    }
}
