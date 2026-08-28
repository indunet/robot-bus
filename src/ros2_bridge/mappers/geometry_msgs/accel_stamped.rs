//! Typed mapper for `geometry_msgs/msg/AccelStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn accel_stamped_to_bus(msg: ros_env::geometry_msgs::msg::AccelStamped) -> crate::geometry_msgs::msg::v1::AccelStamped {
    crate::geometry_msgs::msg::v1::AccelStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        accel: Some(crate::ros2_bridge::mappers::geometry_msgs::accel::accel_to_bus(msg.accel)),
    }
}

pub(crate) fn accel_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::AccelStamped) -> ros_env::geometry_msgs::msg::AccelStamped {
    ros_env::geometry_msgs::msg::AccelStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        accel: crate::ros2_bridge::mappers::geometry_msgs::accel::accel_to_ros(bus.accel.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsAccelStampedMapper;

impl TypedTopicMapper for GeometryMsgsAccelStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::AccelStamped;
    type Bus = crate::geometry_msgs::msg::v1::AccelStamped;

    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/AccelStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(accel_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(accel_stamped_to_ros(msg))
    }
}
