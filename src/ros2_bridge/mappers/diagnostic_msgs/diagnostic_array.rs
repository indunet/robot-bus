//! Typed mapper for `diagnostic_msgs/msg/DiagnosticArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn diagnostic_array_to_bus(msg: ros_env::diagnostic_msgs::msg::DiagnosticArray) -> crate::diagnostic_msgs::msg::v1::DiagnosticArray {
    crate::diagnostic_msgs::msg::v1::DiagnosticArray {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        status: msg.status.into_iter().map(crate::ros2_bridge::mappers::diagnostic_msgs::diagnostic_status::diagnostic_status_to_bus).collect(),
    }
}

pub(crate) fn diagnostic_array_to_ros(bus: crate::diagnostic_msgs::msg::v1::DiagnosticArray) -> ros_env::diagnostic_msgs::msg::DiagnosticArray {
    ros_env::diagnostic_msgs::msg::DiagnosticArray {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        status: bus.status.into_iter().map(crate::ros2_bridge::mappers::diagnostic_msgs::diagnostic_status::diagnostic_status_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DiagnosticMsgsDiagnosticArrayMapper;

impl TypedTopicMapper for DiagnosticMsgsDiagnosticArrayMapper {
    type Ros = ros_env::diagnostic_msgs::msg::DiagnosticArray;
    type Bus = crate::diagnostic_msgs::msg::v1::DiagnosticArray;

    fn type_name(&self) -> &'static str {
        "diagnostic_msgs/msg/DiagnosticArray"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(diagnostic_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(diagnostic_array_to_ros(msg))
    }
}
