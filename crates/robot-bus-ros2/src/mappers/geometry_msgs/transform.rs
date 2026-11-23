//! Typed mapper for `geometry_msgs/msg/Transform`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn transform_to_bus(
    msg: ros_env::geometry_msgs::msg::Transform,
) -> robot_bus::geometry_msgs::msg::v1::Transform {
    robot_bus::geometry_msgs::msg::v1::Transform {
        translation: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.translation),
        ),
        rotation: Some(
            crate::mappers::geometry_msgs::quaternion::quaternion_to_bus(msg.rotation),
        ),
    }
}

pub(crate) fn transform_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Transform,
) -> ros_env::geometry_msgs::msg::Transform {
    ros_env::geometry_msgs::msg::Transform {
        translation: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.translation.unwrap_or_default(),
        ),
        rotation: crate::mappers::geometry_msgs::quaternion::quaternion_to_ros(
            bus.rotation.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsTransformMapper;

impl TypedTopicMapper for GeometryMsgsTransformMapper {
    type Ros = ros_env::geometry_msgs::msg::Transform;
    type Bus = robot_bus::geometry_msgs::msg::v1::Transform;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(transform_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(transform_to_ros(msg))
    }
}
