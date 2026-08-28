//! Typed mapper for `geometry_msgs/msg/PolygonInstanceStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn polygon_instance_stamped_to_bus(msg: ros_env::geometry_msgs::msg::PolygonInstanceStamped) -> crate::geometry_msgs::msg::v1::PolygonInstanceStamped {
    crate::geometry_msgs::msg::v1::PolygonInstanceStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        polygon: Some(crate::ros2_bridge::mappers::geometry_msgs::polygon_instance::polygon_instance_to_bus(msg.polygon)),
    }
}

pub(crate) fn polygon_instance_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::PolygonInstanceStamped) -> ros_env::geometry_msgs::msg::PolygonInstanceStamped {
    ros_env::geometry_msgs::msg::PolygonInstanceStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        polygon: crate::ros2_bridge::mappers::geometry_msgs::polygon_instance::polygon_instance_to_ros(bus.polygon.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsPolygonInstanceStampedMapper;

impl TypedTopicMapper for GeometryMsgsPolygonInstanceStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::PolygonInstanceStamped;
    type Bus = crate::geometry_msgs::msg::v1::PolygonInstanceStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/PolygonInstanceStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(polygon_instance_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(polygon_instance_stamped_to_ros(msg))
    }
}
