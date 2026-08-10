//! Mapper for `sensor_msgs/msg/Imu`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn imu_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::Imu> {
    Ok(crate::sensor_msgs::msg::v1::Imu {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        orientation: nested_view(view, "orientation")?
            .as_ref()
            .map(super::super::geometry_msgs::quaternion::quaternion_from_view)
            .transpose()?,
        orientation_covariance: read_f64_seq(view, "orientation_covariance")?,
        angular_velocity: nested_view(view, "angular_velocity")?
            .as_ref()
            .map(super::super::geometry_msgs::vector3::vector3_from_view)
            .transpose()?,
        angular_velocity_covariance: read_f64_seq(view, "angular_velocity_covariance")?,
        linear_acceleration: nested_view(view, "linear_acceleration")?
            .as_ref()
            .map(super::super::geometry_msgs::vector3::vector3_from_view)
            .transpose()?,
        linear_acceleration_covariance: read_f64_seq(view, "linear_acceleration_covariance")?,
    })
}

pub(crate) fn imu_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::Imu,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.orientation {
        with_nested_mut(view, "orientation", |nested| {
            super::super::geometry_msgs::quaternion::quaternion_write(nested, v)
        })?;
    }
    write_f64_seq(view, "orientation_covariance", &bus.orientation_covariance)?;
    if let Some(v) = &bus.angular_velocity {
        with_nested_mut(view, "angular_velocity", |nested| {
            super::super::geometry_msgs::vector3::vector3_write(nested, v)
        })?;
    }
    write_f64_seq(
        view,
        "angular_velocity_covariance",
        &bus.angular_velocity_covariance,
    )?;
    if let Some(v) = &bus.linear_acceleration {
        with_nested_mut(view, "linear_acceleration", |nested| {
            super::super::geometry_msgs::vector3::vector3_write(nested, v)
        })?;
    }
    write_f64_seq(
        view,
        "linear_acceleration_covariance",
        &bus.linear_acceleration_covariance,
    )?;
    Ok(())
}

pub(crate) fn imu_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::Imu> {
    imu_from_view(&msg.view())
}

pub(crate) fn imu_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::Imu,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/Imu")?;
    imu_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsImuMapper;
impl TopicMapper for SensorMsgsImuMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Imu"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(imu_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::Imu as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/Imu: {e}")))?;
        imu_bus_to_dyn(&bus)
    }
}
