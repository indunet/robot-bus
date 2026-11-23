//! Typed mapper for `geometry_msgs/msg/Vector3Stamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn vector3_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::Vector3Stamped,
) -> robot_bus::geometry_msgs::msg::v1::Vector3Stamped {
    robot_bus::geometry_msgs::msg::v1::Vector3Stamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        vector: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.vector),
        ),
    }
}

pub(crate) fn vector3_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Vector3Stamped,
) -> ros_env::geometry_msgs::msg::Vector3Stamped {
    ros_env::geometry_msgs::msg::Vector3Stamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        vector: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.vector.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsVector3StampedMapper;

impl TypedTopicMapper for GeometryMsgsVector3StampedMapper {
    type Ros = ros_env::geometry_msgs::msg::Vector3Stamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::Vector3Stamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(vector3_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(vector3_stamped_to_ros(msg))
    }
}
