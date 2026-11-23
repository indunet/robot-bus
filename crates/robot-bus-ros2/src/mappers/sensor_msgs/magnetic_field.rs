//! Typed mapper for `sensor_msgs/msg/MagneticField`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn magnetic_field_to_bus(
    msg: ros_env::sensor_msgs::msg::MagneticField,
) -> robot_bus::sensor_msgs::msg::v1::MagneticField {
    robot_bus::sensor_msgs::msg::v1::MagneticField {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        magnetic_field: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(msg.magnetic_field),
        ),
        magnetic_field_covariance: crate::mappers::convert::f64_seq(
            msg.magnetic_field_covariance,
        ),
    }
}

pub(crate) fn magnetic_field_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::MagneticField,
) -> ros_env::sensor_msgs::msg::MagneticField {
    ros_env::sensor_msgs::msg::MagneticField {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        magnetic_field: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.magnetic_field.unwrap_or_default(),
        ),
        magnetic_field_covariance: crate::mappers::convert::FromF64Seq::from_f64_seq(
            bus.magnetic_field_covariance,
        ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsMagneticFieldMapper;

impl TypedTopicMapper for SensorMsgsMagneticFieldMapper {
    type Ros = ros_env::sensor_msgs::msg::MagneticField;
    type Bus = robot_bus::sensor_msgs::msg::v1::MagneticField;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(magnetic_field_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(magnetic_field_to_ros(msg))
    }
}
