//! Typed mapper for `geometry_msgs/msg/PointStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn point_stamped_to_bus(msg: ros_env::geometry_msgs::msg::PointStamped) -> crate::geometry_msgs::msg::v1::PointStamped {
    crate::geometry_msgs::msg::v1::PointStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        point: Some(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus(msg.point)),
    }
}

pub(crate) fn point_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::PointStamped) -> ros_env::geometry_msgs::msg::PointStamped {
    ros_env::geometry_msgs::msg::PointStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        point: crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros(bus.point.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPointStampedMapper;

impl TypedTopicMapper for GeometryMsgsPointStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::PointStamped;
    type Bus = crate::geometry_msgs::msg::v1::PointStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PointStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(point_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(point_stamped_to_ros(msg))
    }
}
