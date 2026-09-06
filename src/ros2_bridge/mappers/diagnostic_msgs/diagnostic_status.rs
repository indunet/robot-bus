//! Typed mapper for `diagnostic_msgs/msg/DiagnosticStatus`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn diagnostic_status_to_bus(
    msg: ros_env::diagnostic_msgs::msg::DiagnosticStatus,
) -> crate::diagnostic_msgs::msg::v1::DiagnosticStatus {
    crate::diagnostic_msgs::msg::v1::DiagnosticStatus {
        level: msg.level.into(),
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        message: crate::ros2_bridge::mappers::convert::from_ros_string(msg.message),
        hardware_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.hardware_id),
        values: msg
            .values
            .into_iter()
            .map(crate::ros2_bridge::mappers::diagnostic_msgs::key_value::key_value_to_bus)
            .collect(),
    }
}

pub(crate) fn diagnostic_status_to_ros(
    bus: crate::diagnostic_msgs::msg::v1::DiagnosticStatus,
) -> ros_env::diagnostic_msgs::msg::DiagnosticStatus {
    ros_env::diagnostic_msgs::msg::DiagnosticStatus {
        level: bus.level as _,
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        message: crate::ros2_bridge::mappers::convert::to_ros_string(bus.message),
        hardware_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.hardware_id),
        values: bus
            .values
            .into_iter()
            .map(crate::ros2_bridge::mappers::diagnostic_msgs::key_value::key_value_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DiagnosticMsgsDiagnosticStatusMapper;

impl TypedTopicMapper for DiagnosticMsgsDiagnosticStatusMapper {
    type Ros = ros_env::diagnostic_msgs::msg::DiagnosticStatus;
    type Bus = crate::diagnostic_msgs::msg::v1::DiagnosticStatus;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(diagnostic_status_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(diagnostic_status_to_ros(msg))
    }
}
