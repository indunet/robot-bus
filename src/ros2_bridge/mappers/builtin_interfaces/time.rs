//! Typed mapper for `builtin_interfaces/msg/Time`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn time_to_bus(msg: ros_env::builtin_interfaces::msg::Time) -> crate::builtin_interfaces::msg::v1::Time {
    crate::builtin_interfaces::msg::v1::Time {
        sec: msg.sec,
        nanosec: msg.nanosec,
    }
}

pub(crate) fn time_to_ros(bus: crate::builtin_interfaces::msg::v1::Time) -> ros_env::builtin_interfaces::msg::Time {
    ros_env::builtin_interfaces::msg::Time {
        sec: bus.sec,
        nanosec: bus.nanosec,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BuiltinInterfacesTimeMapper;

impl TypedTopicMapper for BuiltinInterfacesTimeMapper {
    type Ros = ros_env::builtin_interfaces::msg::Time;
    type Bus = crate::builtin_interfaces::msg::v1::Time;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(time_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(time_to_ros(msg))
    }
}
