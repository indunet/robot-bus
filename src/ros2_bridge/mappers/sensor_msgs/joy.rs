//! Mapper for `sensor_msgs/msg/Joy`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joy_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::Joy> {
    Ok(crate::sensor_msgs::msg::v1::Joy {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        axes: read_f32_seq(view, "axes")?,
        buttons: read_i32_seq(view, "buttons")?,
    })
}

pub(crate) fn joy_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::Joy,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f32_seq(view, "axes", &bus.axes)?;
    write_i32_seq(view, "buttons", &bus.buttons)?;
    Ok(())
}

pub(crate) fn joy_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::Joy> {
    joy_from_view(&msg.view())
}

pub(crate) fn joy_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::Joy,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/Joy")?;
    joy_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsJoyMapper;
impl TopicMapper for SensorMsgsJoyMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Joy"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joy_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::Joy as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/Joy: {e}")))?;
        joy_bus_to_dyn(&bus)
    }
}
