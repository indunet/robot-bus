//! Typed mapper for `builtin_interfaces/msg/Duration`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn duration_to_bus(
    msg: ros_env::builtin_interfaces::msg::Duration,
) -> robot_bus::builtin_interfaces::msg::v1::Duration {
    robot_bus::builtin_interfaces::msg::v1::Duration {
        sec: msg.sec.into(),
        nanosec: msg.nanosec.into(),
    }
}

pub(crate) fn duration_to_ros(
    bus: robot_bus::builtin_interfaces::msg::v1::Duration,
) -> ros_env::builtin_interfaces::msg::Duration {
    ros_env::builtin_interfaces::msg::Duration {
        sec: bus.sec as _,
        nanosec: bus.nanosec as _,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BuiltinInterfacesDurationMapper;

impl TypedTopicMapper for BuiltinInterfacesDurationMapper {
    type Ros = ros_env::builtin_interfaces::msg::Duration;
    type Bus = robot_bus::builtin_interfaces::msg::v1::Duration;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(duration_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(duration_to_ros(msg))
    }
}
