//! Mapper for `geometry_msgs/msg/AccelWithCovarianceStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn accel_with_covariance_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::AccelWithCovarianceStamped> {
    Ok(crate::geometry_msgs::msg::v1::AccelWithCovarianceStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        accel: nested_view(view, "accel")?
            .as_ref()
            .map(super::accel_with_covariance::accel_with_covariance_from_view)
            .transpose()?,
    })
}

pub(crate) fn accel_with_covariance_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::AccelWithCovarianceStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.accel {
        with_nested_mut(view, "accel", |nested| {
            super::accel_with_covariance::accel_with_covariance_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn accel_with_covariance_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::AccelWithCovarianceStamped> {
    accel_with_covariance_stamped_from_view(&msg.view())
}

pub(crate) fn accel_with_covariance_stamped_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::AccelWithCovarianceStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/AccelWithCovarianceStamped")?;
    accel_with_covariance_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsAccelWithCovarianceStampedMapper;
impl TopicMapper for GeometryMsgsAccelWithCovarianceStampedMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/AccelWithCovarianceStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(accel_with_covariance_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::geometry_msgs::msg::v1::AccelWithCovarianceStamped as ProstMessage>::decode(
                payload,
            )
            .map_err(|e| {
                BusError::Protocol(format!(
                    "decode geometry_msgs/msg/AccelWithCovarianceStamped: {e}"
                ))
            })?;
        accel_with_covariance_stamped_bus_to_dyn(&bus)
    }
}
