//! Typed mapper for `geometry_msgs/msg/Wrench`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn wrench_to_bus(msg: ros_env::geometry_msgs::msg::Wrench) -> crate::geometry_msgs::msg::v1::Wrench {
    crate::geometry_msgs::msg::v1::Wrench {
        force: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.force)),
        torque: Some(crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_bus(msg.torque)),
    }
}

pub(crate) fn wrench_to_ros(bus: crate::geometry_msgs::msg::v1::Wrench) -> ros_env::geometry_msgs::msg::Wrench {
    ros_env::geometry_msgs::msg::Wrench {
        force: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.force.unwrap_or_default()),
        torque: crate::ros2_bridge::mappers::geometry_msgs::vector3::vector3_to_ros(bus.torque.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsWrenchMapper;

impl TypedTopicMapper for GeometryMsgsWrenchMapper {
    type Ros = ros_env::geometry_msgs::msg::Wrench;
    type Bus = crate::geometry_msgs::msg::v1::Wrench;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(wrench_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(wrench_to_ros(msg))
    }
}
