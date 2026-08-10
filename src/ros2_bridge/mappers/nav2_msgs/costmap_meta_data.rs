//! Mapper for `nav2_msgs/msg/CostmapMetaData`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn costmap_meta_data_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::CostmapMetaData> {
    Ok(crate::nav2_msgs::msg::v1::CostmapMetaData {
        map_load_time: nested_view(view, "map_load_time")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
        update_time: nested_view(view, "update_time")?
            .as_ref()
            .map(super::super::builtin_interfaces::time::time_from_view)
            .transpose()?,
        resolution: read_f32(view, "resolution")?,
        size_x: read_u32(view, "size_x")?,
        size_y: read_u32(view, "size_y")?,
        origin: nested_view(view, "origin")?
            .as_ref()
            .map(super::super::geometry_msgs::pose::pose_from_view)
            .transpose()?,
        layer: read_string(view, "layer")?,
    })
}

pub(crate) fn costmap_meta_data_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::CostmapMetaData,
) -> Result<()> {
    if let Some(v) = &bus.map_load_time {
        with_nested_mut(view, "map_load_time", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.update_time {
        with_nested_mut(view, "update_time", |nested| {
            super::super::builtin_interfaces::time::time_write(nested, v)
        })?;
    }
    write_f32(view, "resolution", bus.resolution)?;
    write_u32(view, "size_x", bus.size_x)?;
    write_u32(view, "size_y", bus.size_y)?;
    if let Some(v) = &bus.origin {
        with_nested_mut(view, "origin", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    write_string(view, "layer", &bus.layer)?;
    Ok(())
}

pub(crate) fn costmap_meta_data_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::CostmapMetaData> {
    costmap_meta_data_from_view(&msg.view())
}

pub(crate) fn costmap_meta_data_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::CostmapMetaData,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/CostmapMetaData")?;
    costmap_meta_data_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsCostmapMetaDataMapper;
impl TopicMapper for Nav2MsgsCostmapMetaDataMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/CostmapMetaData"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(costmap_meta_data_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::CostmapMetaData as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode nav2_msgs/msg/CostmapMetaData: {e}"))
            })?;
        costmap_meta_data_bus_to_dyn(&bus)
    }
}
