//! Mapper for `sensor_msgs/msg/Image`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn image_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::Image> {
    Ok(crate::sensor_msgs::msg::v1::Image {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        height: read_u32(view, "height")?,
        width: read_u32(view, "width")?,
        encoding: read_string(view, "encoding")?,
        is_bigendian: read_bool(view, "is_bigendian")?,
        step: read_u32(view, "step")?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn image_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::Image,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32(view, "height", bus.height)?;
    write_u32(view, "width", bus.width)?;
    write_string(view, "encoding", &bus.encoding)?;
    write_bool(view, "is_bigendian", bus.is_bigendian)?;
    write_u32(view, "step", bus.step)?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn image_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::Image> {
    image_from_view(&msg.view())
}

pub(crate) fn image_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::Image,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/Image")?;
    image_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsImageMapper;
impl TopicMapper for SensorMsgsImageMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/Image"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(image_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::Image as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/Image: {e}")))?;
        image_bus_to_dyn(&bus)
    }
}
