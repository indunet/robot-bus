//! Typed mapper for `geometry_msgs/msg/Wrench`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn wrench_to_bus(
    msg: ros_env::geometry_msgs::msg::Wrench,
) -> robot_bus::geometry_msgs::msg::v1::Wrench {
    robot_bus::geometry_msgs::msg::v1::Wrench {
        force: Some(crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.force)),
        torque: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.torque),
        ),
    }
}

pub(crate) fn wrench_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Wrench,
) -> ros_env::geometry_msgs::msg::Wrench {
    ros_env::geometry_msgs::msg::Wrench {
        force: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.force.unwrap_or_default(),
        ),
        torque: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.torque.unwrap_or_default(),
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsWrenchMapper;

impl TypedTopicMapper for GeometryMsgsWrenchMapper {
    type Ros = ros_env::geometry_msgs::msg::Wrench;
    type Bus = robot_bus::geometry_msgs::msg::v1::Wrench;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(wrench_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(wrench_to_ros(msg))
    }
}
