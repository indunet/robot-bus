//! Mapper for `nav_msgs/msg/MapMetaData`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn map_meta_data_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::MapMetaData> {
    Ok(crate::nav_msgs::msg::v1::MapMetaData {
        map_load_time: nested_view(view, "map_load_time")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
        resolution: read_f32(view, "resolution")?,
        width: read_u32(view, "width")?,
        height: read_u32(view, "height")?,
        origin: nested_view(view, "origin")?
            .as_ref()
            .map(super::super::geometry_msgs::pose::pose_from_view)
            .transpose()?,
    })
}

pub(crate) fn map_meta_data_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::MapMetaData,
) -> Result<()> {
    if let Some(v) = &bus.map_load_time {
        with_nested_mut(view, "map_load_time", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    write_f32(view, "resolution", bus.resolution)?;
    write_u32(view, "width", bus.width)?;
    write_u32(view, "height", bus.height)?;
    if let Some(v) = &bus.origin {
        with_nested_mut(view, "origin", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    Ok(())
}

pub(crate) fn map_meta_data_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::MapMetaData> {
    map_meta_data_from_view(&msg.view())
}

pub(crate) fn map_meta_data_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::MapMetaData,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/MapMetaData")?;
    map_meta_data_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsMapMetaDataMapper;
impl TopicMapper for NavMsgsMapMetaDataMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/MapMetaData"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(map_meta_data_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::MapMetaData as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/MapMetaData: {e}")))?;
        map_meta_data_bus_to_dyn(&bus)
    }
}
