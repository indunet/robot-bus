//! Mapper for `geometry_msgs/msg/TwistWithCovariance`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn twist_with_covariance_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::TwistWithCovariance> {
    Ok(crate::geometry_msgs::msg::v1::TwistWithCovariance {
        twist: nested_view(view, "twist")?
            .as_ref()
            .map(super::twist::twist_from_view)
            .transpose()?,
        covariance: read_f64_seq(view, "covariance")?,
    })
}

pub(crate) fn twist_with_covariance_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::TwistWithCovariance,
) -> Result<()> {
    if let Some(v) = &bus.twist {
        with_nested_mut(view, "twist", |nested| super::twist::twist_write(nested, v))?;
    }
    write_f64_seq(view, "covariance", &bus.covariance)?;
    Ok(())
}

pub(crate) fn twist_with_covariance_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::TwistWithCovariance> {
    twist_with_covariance_from_view(&msg.view())
}

pub(crate) fn twist_with_covariance_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::TwistWithCovariance,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/TwistWithCovariance")?;
    twist_with_covariance_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsTwistWithCovarianceMapper;
impl TopicMapper for GeometryMsgsTwistWithCovarianceMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/TwistWithCovariance"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(twist_with_covariance_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::geometry_msgs::msg::v1::TwistWithCovariance as ProstMessage>::decode(payload)
                .map_err(|e| {
                BusError::Protocol(format!("decode geometry_msgs/msg/TwistWithCovariance: {e}"))
            })?;
        twist_with_covariance_bus_to_dyn(&bus)
    }
}
