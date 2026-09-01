//! Typed mapper for `geometry_msgs/msg/WrenchStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn wrench_stamped_to_bus(msg: ros_env::geometry_msgs::msg::WrenchStamped) -> crate::geometry_msgs::msg::v1::WrenchStamped {
    crate::geometry_msgs::msg::v1::WrenchStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        wrench: Some(crate::ros2_bridge::mappers::geometry_msgs::wrench::wrench_to_bus(msg.wrench)),
    }
}

pub(crate) fn wrench_stamped_to_ros(bus: crate::geometry_msgs::msg::v1::WrenchStamped) -> ros_env::geometry_msgs::msg::WrenchStamped {
    ros_env::geometry_msgs::msg::WrenchStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        wrench: crate::ros2_bridge::mappers::geometry_msgs::wrench::wrench_to_ros(bus.wrench.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsWrenchStampedMapper;

impl TypedTopicMapper for GeometryMsgsWrenchStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::WrenchStamped;
    type Bus = crate::geometry_msgs::msg::v1::WrenchStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(wrench_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(wrench_stamped_to_ros(msg))
    }
}
