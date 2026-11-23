//! Typed mapper for `sensor_msgs/msg/Imu`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn imu_to_bus(msg: ros_env::sensor_msgs::msg::Imu) -> robot_bus::sensor_msgs::msg::v1::Imu {
    robot_bus::sensor_msgs::msg::v1::Imu {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        orientation: Some(
            crate::mappers::geometry_msgs::quaternion::quaternion_to_bus(
                msg.orientation,
            ),
        ),
        orientation_covariance: crate::mappers::convert::f64_seq(
            msg.orientation_covariance,
        ),
        angular_velocity: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(
                msg.angular_velocity,
            ),
        ),
        angular_velocity_covariance: crate::mappers::convert::f64_seq(
            msg.angular_velocity_covariance,
        ),
        linear_acceleration: Some(
            crate::mappers::geometry_msgs::vector3::vector3_to_bus(
                msg.linear_acceleration,
            ),
        ),
        linear_acceleration_covariance: crate::mappers::convert::f64_seq(
            msg.linear_acceleration_covariance,
        ),
    }
}

pub(crate) fn imu_to_ros(bus: robot_bus::sensor_msgs::msg::v1::Imu) -> ros_env::sensor_msgs::msg::Imu {
    ros_env::sensor_msgs::msg::Imu {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        orientation: crate::mappers::geometry_msgs::quaternion::quaternion_to_ros(
            bus.orientation.unwrap_or_default(),
        ),
        orientation_covariance: crate::mappers::convert::FromF64Seq::from_f64_seq(
            bus.orientation_covariance,
        ),
        angular_velocity: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.angular_velocity.unwrap_or_default(),
        ),
        angular_velocity_covariance: crate::mappers::convert::FromF64Seq::from_f64_seq(
            bus.angular_velocity_covariance,
        ),
        linear_acceleration: crate::mappers::geometry_msgs::vector3::vector3_to_ros(
            bus.linear_acceleration.unwrap_or_default(),
        ),
        linear_acceleration_covariance:
            crate::mappers::convert::FromF64Seq::from_f64_seq(
                bus.linear_acceleration_covariance,
            ),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsImuMapper;

impl TypedTopicMapper for SensorMsgsImuMapper {
    type Ros = ros_env::sensor_msgs::msg::Imu;
    type Bus = robot_bus::sensor_msgs::msg::v1::Imu;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(imu_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(imu_to_ros(msg))
    }
}
