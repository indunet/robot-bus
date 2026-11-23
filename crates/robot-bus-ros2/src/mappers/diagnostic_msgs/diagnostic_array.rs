//! Typed mapper for `diagnostic_msgs/msg/DiagnosticArray`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn diagnostic_array_to_bus(
    msg: ros_env::diagnostic_msgs::msg::DiagnosticArray,
) -> robot_bus::diagnostic_msgs::msg::v1::DiagnosticArray {
    robot_bus::diagnostic_msgs::msg::v1::DiagnosticArray {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        status: msg.status.into_iter().map(crate::mappers::diagnostic_msgs::diagnostic_status::diagnostic_status_to_bus).collect(),
    }
}

pub(crate) fn diagnostic_array_to_ros(
    bus: robot_bus::diagnostic_msgs::msg::v1::DiagnosticArray,
) -> ros_env::diagnostic_msgs::msg::DiagnosticArray {
    ros_env::diagnostic_msgs::msg::DiagnosticArray {
        header: crate::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        status: bus.status.into_iter().map(crate::mappers::diagnostic_msgs::diagnostic_status::diagnostic_status_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DiagnosticMsgsDiagnosticArrayMapper;

impl TypedTopicMapper for DiagnosticMsgsDiagnosticArrayMapper {
    type Ros = ros_env::diagnostic_msgs::msg::DiagnosticArray;
    type Bus = robot_bus::diagnostic_msgs::msg::v1::DiagnosticArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(diagnostic_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(diagnostic_array_to_ros(msg))
    }
}
