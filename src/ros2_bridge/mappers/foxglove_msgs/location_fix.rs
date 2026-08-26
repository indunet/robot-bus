//! Mapper for `foxglove_msgs/msg/LocationFix`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn location_fix_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::LocationFix> {
    Ok(crate::foxglove_msgs::msg::v1::LocationFix {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        latitude: read_f64(view, "latitude")?,
        longitude: read_f64(view, "longitude")?,
        altitude: read_f64(view, "altitude")?,
        position_covariance: read_f64_seq(view, "position_covariance")?,
        position_covariance_type: read_i32(view, "position_covariance_type")?,
        heading: read_f64_opt(view, "heading")?,
        velocity: nested_view(view, "velocity")?
            .as_ref()
            .map(super::vector3::vector3_from_view)
            .transpose()?,
        color: nested_view(view, "color")?
            .as_ref()
            .map(super::color::color_from_view)
            .transpose()?,
        metadata: read_message_seq(
            view,
            "metadata",
            super::key_value_pair::key_value_pair_from_view,
        )?,
    })
}

pub(crate) fn location_fix_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::LocationFix,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    write_f64(view, "latitude", bus.latitude)?;
    write_f64(view, "longitude", bus.longitude)?;
    write_f64(view, "altitude", bus.altitude)?;
    write_f64_seq(view, "position_covariance", &bus.position_covariance)?;
    write_i32(
        view,
        "position_covariance_type",
        bus.position_covariance_type,
    )?;
    if let Some(v) = bus.heading {
        write_f64(view, "heading", v)?;
    }
    if let Some(v) = &bus.velocity {
        with_nested_mut(view, "velocity", |nested| {
            super::vector3::vector3_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.color {
        with_nested_mut(view, "color", |nested| super::color::color_write(nested, v))?;
    }
    write_message_seq(
        view,
        "metadata",
        &bus.metadata,
        super::key_value_pair::key_value_pair_write,
    )?;
    Ok(())
}

pub(crate) fn location_fix_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::LocationFix> {
    location_fix_from_view(&msg.view())
}

pub(crate) fn location_fix_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::LocationFix,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/LocationFix")?;
    location_fix_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsLocationFixMapper;
impl TopicMapper for FoxgloveMsgsLocationFixMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/LocationFix"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(location_fix_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::LocationFix as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/LocationFix: {e}"))
            })?;
        location_fix_bus_to_dyn(&bus)
    }
}
