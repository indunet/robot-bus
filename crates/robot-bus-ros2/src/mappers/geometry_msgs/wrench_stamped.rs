//! Typed mapper for `geometry_msgs/msg/WrenchStamped`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn wrench_stamped_to_bus(
    msg: ros_env::geometry_msgs::msg::WrenchStamped,
) -> robot_bus::geometry_msgs::msg::v1::WrenchStamped {
    robot_bus::geometry_msgs::msg::v1::WrenchStamped {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        wrench: Some(crate::mappers::geometry_msgs::wrench::wrench_to_bus(msg.wrench)),
    }
}

pub(crate) fn wrench_stamped_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::WrenchStamped,
) -> ros_env::geometry_msgs::msg::WrenchStamped {
    ros_env::geometry_msgs::msg::WrenchStamped {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        wrench: crate::mappers::geometry_msgs::wrench::wrench_to_ros(
            bus.wrench.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsWrenchStampedMapper;

impl TypedTopicMapper for GeometryMsgsWrenchStampedMapper {
    type Ros = ros_env::geometry_msgs::msg::WrenchStamped;
    type Bus = robot_bus::geometry_msgs::msg::v1::WrenchStamped;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(wrench_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(wrench_stamped_to_ros(msg))
    }
}
