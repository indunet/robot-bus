//! Mapper for `foxglove_msgs/msg/PointCloud`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn point_cloud_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::PointCloud> {
    Ok(crate::foxglove_msgs::msg::v1::PointCloud {
        timestamp: read_timestamp(view, "timestamp")?,
        frame_id: read_string(view, "frame_id")?,
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::pose::pose_from_view)
            .transpose()?,
        point_stride: read_u32(view, "point_stride")?,
        fields: read_message_seq(
            view,
            "fields",
            super::packed_element_field::packed_element_field_from_view,
        )?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn point_cloud_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::PointCloud,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_string(view, "frame_id", &bus.frame_id)?;
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| super::pose::pose_write(nested, v))?;
    }
    write_u32(view, "point_stride", bus.point_stride)?;
    write_message_seq(
        view,
        "fields",
        &bus.fields,
        super::packed_element_field::packed_element_field_write,
    )?;
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn point_cloud_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::PointCloud> {
    point_cloud_from_view(&msg.view())
}

pub(crate) fn point_cloud_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::PointCloud,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/PointCloud")?;
    point_cloud_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsPointCloudMapper;
impl TopicMapper for FoxgloveMsgsPointCloudMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/PointCloud"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(point_cloud_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::PointCloud as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode foxglove_msgs/msg/PointCloud: {e}")))?;
        point_cloud_bus_to_dyn(&bus)
    }
}
