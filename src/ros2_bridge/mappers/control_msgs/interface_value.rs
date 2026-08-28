//! Typed mapper for `control_msgs/msg/InterfaceValue`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn interface_value_to_bus(msg: ros_env::control_msgs::msg::InterfaceValue) -> crate::control_msgs::msg::v1::InterfaceValue {
    crate::control_msgs::msg::v1::InterfaceValue {
        interface_names: crate::ros2_bridge::mappers::convert::string_seq(msg.interface_names),
        values: crate::ros2_bridge::mappers::convert::f64_seq(msg.values),
    }
}

pub(crate) fn interface_value_to_ros(bus: crate::control_msgs::msg::v1::InterfaceValue) -> ros_env::control_msgs::msg::InterfaceValue {
    ros_env::control_msgs::msg::InterfaceValue {
        interface_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.interface_names),
        values: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.values),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsInterfaceValueMapper;

impl TypedTopicMapper for ControlMsgsInterfaceValueMapper {
    type Ros = ros_env::control_msgs::msg::InterfaceValue;
    type Bus = crate::control_msgs::msg::v1::InterfaceValue;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/InterfaceValue"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(interface_value_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(interface_value_to_ros(msg))
    }
}
