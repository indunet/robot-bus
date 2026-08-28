//! Typed mapper for `geometry_msgs/msg/Vector3Stamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn vector3_stamped_to_bus(msg: ros_env::geometry_msgs::msg::Vector3Stamped) -> crate::geometry_msgs::msg::v1::Vector3Stamped {
    crate::geometry_msgs::msg::v1::Vector3Stamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        vector: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.vector)),
    }
}

pub(crate) fn vector3_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::Vector3Stamped) -> ros_env::geometry_msgs::msg::Vector3Stamped {
    ros_env::geometry_msgs::msg::Vector3Stamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        vector: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.vector.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsVector3StampedMapper;

impl TypedTopicMapper for GeometryMsgsVector3StampedMapper {
    type Ros = ros_env::geometry_msgs::msg::Vector3Stamped;
    type Bus = crate::geometry_msgs::msg::v1::Vector3Stamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Vector3Stamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(vector3_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(vector3_stamped_to_ros(msg))
    }
}
