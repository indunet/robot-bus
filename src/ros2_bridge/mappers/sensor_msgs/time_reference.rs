//! Typed mapper for `sensor_msgs/msg/TimeReference`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn time_reference_to_bus(msg: ros_env::sensor_msgs::msg::TimeReference) -> crate::sensor_msgs::msg::v1::TimeReference {
    crate::sensor_msgs::msg::v1::TimeReference {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        time_ref: Some(crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_bus(msg.time_ref)),
        source: crate::ros2_bridge::mappers::convert::from_ros_string(msg.source),
    }
}

pub(crate) fn time_reference_to_ros(bus: crate::sensor_msgs::msg::v1::TimeReference) -> ros_env::sensor_msgs::msg::TimeReference {
    ros_env::sensor_msgs::msg::TimeReference {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        time_ref: crate::ros2_bridge::mappers::builtin_interfaces::time::time_to_ros(bus.time_ref.unwrap_or_default()),
        source: crate::ros2_bridge::mappers::convert::to_ros_string(bus.source),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsTimeReferenceMapper;

impl TypedTopicMapper for SensorMsgsTimeReferenceMapper {
    type Ros = ros_env::sensor_msgs::msg::TimeReference;
    type Bus = crate::sensor_msgs::msg::v1::TimeReference;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(time_reference_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(time_reference_to_ros(msg))
    }
}
