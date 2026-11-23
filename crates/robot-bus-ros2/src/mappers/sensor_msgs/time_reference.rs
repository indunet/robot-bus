//! Typed mapper for `sensor_msgs/msg/TimeReference`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn time_reference_to_bus(
    msg: ros_env::sensor_msgs::msg::TimeReference,
) -> robot_bus::sensor_msgs::msg::v1::TimeReference {
    robot_bus::sensor_msgs::msg::v1::TimeReference {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        time_ref: Some(
            crate::mappers::builtin_interfaces::time::time_to_bus(msg.time_ref),
        ),
        source: crate::mappers::convert::from_ros_string(msg.source),
    }
}

pub(crate) fn time_reference_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::TimeReference,
) -> ros_env::sensor_msgs::msg::TimeReference {
    ros_env::sensor_msgs::msg::TimeReference {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        time_ref: crate::mappers::builtin_interfaces::time::time_to_ros(
            bus.time_ref.unwrap_or_default(),
        ),
        source: crate::mappers::convert::to_ros_string(bus.source),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsTimeReferenceMapper;

impl TypedTopicMapper for SensorMsgsTimeReferenceMapper {
    type Ros = ros_env::sensor_msgs::msg::TimeReference;
    type Bus = robot_bus::sensor_msgs::msg::v1::TimeReference;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(time_reference_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(time_reference_to_ros(msg))
    }
}
