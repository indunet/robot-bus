//! Mapper for `sensor_msgs/msg/Range`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn range_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::Range> {
    Ok(crate::sensor_msgs::msg::v1::Range {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        radiation_type: read_u32(view, "radiation_type")?,
        field_of_view: read_f32(view, "field_of_view")?,
        min_range: read_f32(view, "min_range")?,
        max_range: read_f32(view, "max_range")?,
        range: read_f32(view, "range")?,
    })
}

pub(crate) fn range_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::Range,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32(view, "radiation_type", bus.radiation_type)?;
    write_f32(view, "field_of_view", bus.field_of_view)?;
    write_f32(view, "min_range", bus.min_range)?;
    write_f32(view, "max_range", bus.max_range)?;
    write_f32(view, "range", bus.range)?;
    Ok(())
}

pub(crate) fn range_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::Range> {
    range_from_view(&msg.view())
}

pub(crate) fn range_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::Range,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/Range")?;
    range_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsRangeMapper;
impl TopicMapper for SensorMsgsRangeMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Range"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(range_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::Range as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/Range: {e}")))?;
        range_bus_to_dyn(&bus)
    }
}
