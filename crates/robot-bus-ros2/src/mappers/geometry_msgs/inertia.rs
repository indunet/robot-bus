//! Typed mapper for `geometry_msgs/msg/Inertia`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn inertia_to_bus(
    msg: ros_env::geometry_msgs::msg::Inertia,
) -> robot_bus::geometry_msgs::msg::v1::Inertia {
    robot_bus::geometry_msgs::msg::v1::Inertia {
        m: msg.m,
        com: Some(crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.com)),
        ixx: msg.ixx,
        ixy: msg.ixy,
        ixz: msg.ixz,
        iyy: msg.iyy,
        iyz: msg.iyz,
        izz: msg.izz,
    }
}

pub(crate) fn inertia_to_ros(
    bus: robot_bus::geometry_msgs::msg::v1::Inertia,
) -> ros_env::geometry_msgs::msg::Inertia {
    ros_env::geometry_msgs::msg::Inertia {
        m: bus.m,
        com: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.com.unwrap_or_default(),
        ),
        ixx: bus.ixx,
        ixy: bus.ixy,
        ixz: bus.ixz,
        iyy: bus.iyy,
        iyz: bus.iyz,
        izz: bus.izz,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GeometryMsgsInertiaMapper;

impl TypedTopicMapper for GeometryMsgsInertiaMapper {
    type Ros = ros_env::geometry_msgs::msg::Inertia;
    type Bus = robot_bus::geometry_msgs::msg::v1::Inertia;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(inertia_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(inertia_to_ros(msg))
    }
}
