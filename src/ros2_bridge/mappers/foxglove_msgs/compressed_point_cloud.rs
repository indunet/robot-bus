//! Mapper for `foxglove_msgs/msg/CompressedPointCloud`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn compressed_point_cloud_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedPointCloud> {
    Ok(crate::foxglove_msgs::msg::v1::CompressedPointCloud {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        data: read_byte_seq(view, "data")?,
        format: read_string(view, "format")?,
    })
}

pub(crate) fn compressed_point_cloud_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::CompressedPointCloud,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_byte_seq(view, "data", &bus.data)?;
    write_string(view, "format", &bus.format)?;
    Ok(())
}

pub(crate) fn compressed_point_cloud_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::CompressedPointCloud> {
    compressed_point_cloud_from_view(&msg.view())
}

pub(crate) fn compressed_point_cloud_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::CompressedPointCloud,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/CompressedPointCloud")?;
    compressed_point_cloud_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsCompressedPointCloudMapper;
impl TopicMapper for FoxgloveMsgsCompressedPointCloudMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/CompressedPointCloud"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(compressed_point_cloud_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::CompressedPointCloud as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!(
                        "decode foxglove_msgs/msg/CompressedPointCloud: {e}"
                    ))
                })?;
        compressed_point_cloud_bus_to_dyn(&bus)
    }
}
