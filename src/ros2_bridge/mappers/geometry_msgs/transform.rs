//! Typed mapper for `geometry_msgs/msg/Transform`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn transform_to_bus(msg: ros_env::geometry_msgs::msg::Transform) -> crate::geometry_msgs::msg::v1::Transform {
    crate::geometry_msgs::msg::v1::Transform {
        translation: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.translation)),
        rotation: Some(crate::ros2_bridge::mappers::geometry_msgs::quaternion::quaternion_to_bus(msg.rotation)),
    }
}

pub(crate) fn transform_to_ros(bus: crate::geometry_msgs::msg::v1::Transform) -> ros_env::geometry_msgs::msg::Transform {
    ros_env::geometry_msgs::msg::Transform {
        translation: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.translation.unwrap_or_default()),
        rotation: crate::ros2_bridge::mappers::geometry_msgs::quaternion::quaternion_to_ros(bus.rotation.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTransformMapper;

impl TypedTopicMapper for GeometryMsgsTransformMapper {
    type Ros = ros_env::geometry_msgs::msg::Transform;
    type Bus = crate::geometry_msgs::msg::v1::Transform;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(transform_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(transform_to_ros(msg))
    }
}
