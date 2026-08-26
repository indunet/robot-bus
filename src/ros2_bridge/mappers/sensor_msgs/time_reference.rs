//! Mapper for `sensor_msgs/msg/TimeReference`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn time_reference_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::TimeReference> {
    Ok(crate::sensor_msgs::msg::v1::TimeReference {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        time_ref: nested_view(view, "time_ref")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
        source: read_string(view, "source")?,
    })
}

pub(crate) fn time_reference_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::TimeReference,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.time_ref {
        with_nested_mut(view, "time_ref", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    write_string(view, "source", &bus.source)?;
    Ok(())
}

pub(crate) fn time_reference_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::TimeReference> {
    time_reference_from_view(&msg.view())
}

pub(crate) fn time_reference_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::TimeReference,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/TimeReference")?;
    time_reference_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsTimeReferenceMapper;
impl TopicMapper for SensorMsgsTimeReferenceMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/TimeReference"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(time_reference_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::TimeReference as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode sensor_msgs/msg/TimeReference: {e}"))
            })?;
        time_reference_bus_to_dyn(&bus)
    }
}
