//! Typed mapper for `builtin_interfaces/msg/Time`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn time_to_bus(
    msg: ros_env::builtin_interfaces::msg::Time,
) -> robot_bus::builtin_interfaces::msg::v1::Time {
    robot_bus::builtin_interfaces::msg::v1::Time {
        sec: msg.sec.into(),
        nanosec: msg.nanosec.into(),
    }
}

pub(crate) fn time_to_ros(
    bus: robot_bus::builtin_interfaces::msg::v1::Time,
) -> ros_env::builtin_interfaces::msg::Time {
    ros_env::builtin_interfaces::msg::Time {
        sec: bus.sec as _,
        nanosec: bus.nanosec as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BuiltinInterfacesTimeMapper;

impl TypedTopicMapper for BuiltinInterfacesTimeMapper {
    type Ros = ros_env::builtin_interfaces::msg::Time;
    type Bus = robot_bus::builtin_interfaces::msg::v1::Time;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(time_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(time_to_ros(msg))
    }
}
