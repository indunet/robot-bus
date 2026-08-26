//! Mapper for `sensor_msgs/msg/PointCloud`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn point_cloud_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::PointCloud> {
    Ok(crate::sensor_msgs::msg::v1::PointCloud {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        points: read_message_seq(
            view,
            "points",
            super::super::geometry_msgs::point32::point32_from_view,
        )?,
        channels: read_message_seq(
            view,
            "channels",
            super::channel_float32::channel_float32_from_view,
        )?,
    })
}

pub(crate) fn point_cloud_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::PointCloud,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "points",
        &bus.points,
        super::super::geometry_msgs::point32::point32_write,
    )?;
    write_message_seq(
        view,
        "channels",
        &bus.channels,
        super::channel_float32::channel_float32_write,
    )?;
    Ok(())
}

pub(crate) fn point_cloud_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::PointCloud> {
    point_cloud_from_view(&msg.view())
}

pub(crate) fn point_cloud_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::PointCloud,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/PointCloud")?;
    point_cloud_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsPointCloudMapper;
impl TopicMapper for SensorMsgsPointCloudMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/PointCloud"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point_cloud_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::PointCloud as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/PointCloud: {e}")))?;
        point_cloud_bus_to_dyn(&bus)
    }
}
