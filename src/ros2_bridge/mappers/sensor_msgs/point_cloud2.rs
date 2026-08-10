//! Mapper for `sensor_msgs/msg/PointCloud2`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn point_cloud2_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::PointCloud2> {
    Ok(crate::sensor_msgs::msg::v1::PointCloud2 {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        height: read_u32(view, "height")?,
        width: read_u32(view, "width")?,
        fields: read_message_seq(view, "fields", super::point_field::point_field_from_view)?,
        is_bigendian: read_bool(view, "is_bigendian")?,
        point_step: read_u32(view, "point_step")?,
        row_step: read_u32(view, "row_step")?,
        data: read_byte_seq(view, "data")?,
        is_dense: read_bool(view, "is_dense")?,
    })
}

pub(crate) fn point_cloud2_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::PointCloud2,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32(view, "height", bus.height)?;
    write_u32(view, "width", bus.width)?;
    write_message_seq(view, "fields", &bus.fields, super::point_field::point_field_write)?;
    write_bool(view, "is_bigendian", bus.is_bigendian)?;
    write_u32(view, "point_step", bus.point_step)?;
    write_u32(view, "row_step", bus.row_step)?;
    write_byte_seq(view, "data", &bus.data)?;
    write_bool(view, "is_dense", bus.is_dense)?;
    Ok(())
}

pub(crate) fn point_cloud2_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::PointCloud2> {
    point_cloud2_from_view(&msg.view())
}

pub(crate) fn point_cloud2_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::PointCloud2,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/PointCloud2")?;
    point_cloud2_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsPointCloud2Mapper;
impl TopicMapper for SensorMsgsPointCloud2Mapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/PointCloud2"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point_cloud2_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::PointCloud2 as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/PointCloud2: {e}")))?;
        point_cloud2_bus_to_dyn(&bus)
    }
}
