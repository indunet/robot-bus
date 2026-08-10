//! Mapper for `sensor_msgs/msg/ChannelFloat32`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn channel_float32_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::ChannelFloat32> {
    Ok(crate::sensor_msgs::msg::v1::ChannelFloat32 {
        name: read_string(view, "name")?,
        values: read_f32_seq(view, "values")?,
    })
}

pub(crate) fn channel_float32_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::ChannelFloat32,
) -> Result<()> {
    write_string(view, "name", &bus.name)?;
    write_f32_seq(view, "values", &bus.values)?;
    Ok(())
}

pub(crate) fn channel_float32_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::ChannelFloat32> {
    channel_float32_from_view(&msg.view())
}

pub(crate) fn channel_float32_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::ChannelFloat32,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/ChannelFloat32")?;
    channel_float32_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsChannelFloat32Mapper;
impl TopicMapper for SensorMsgsChannelFloat32Mapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/ChannelFloat32"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(channel_float32_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::ChannelFloat32 as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode sensor_msgs/msg/ChannelFloat32: {e}"))
            })?;
        channel_float32_bus_to_dyn(&bus)
    }
}
