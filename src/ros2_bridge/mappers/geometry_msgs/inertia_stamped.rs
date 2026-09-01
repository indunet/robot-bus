//! Typed mapper for `geometry_msgs/msg/InertiaStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn inertia_stamped_to_bus(msg: ros_env::geometry_msgs::msg::InertiaStamped) -> crate::geometry_msgs::msg::v1::InertiaStamped {
    crate::geometry_msgs::msg::v1::InertiaStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        inertia: Some(crate::ros2_bridge::mappers::geometry_msgs::inertia::inertia_to_bus(msg.inertia)),
    }
}

pub(crate) fn inertia_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::InertiaStamped) -> ros_env::geometry_msgs::msg::InertiaStamped {
    ros_env::geometry_msgs::msg::InertiaStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        inertia: crate::ros2_bridge::mappers::geometry_msgs::inertia::inertia_to_ros(bus.inertia.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsInertiaStampedMapper;

impl TypedTopicMapper for GeometryMsgsInertiaStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::InertiaStamped;
    type Bus = crate::geometry_msgs::msg::v1::InertiaStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(inertia_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(inertia_stamped_to_ros(msg))
    }
}
