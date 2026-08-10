//! Mapper for `nav2_msgs/msg/CostmapFilterInfo`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn costmap_filter_info_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::CostmapFilterInfo> {
    Ok(crate::nav2_msgs::msg::v1::CostmapFilterInfo {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        r#type: read_u32(view, "type")?,
        filter_mask_topic: read_string(view, "filter_mask_topic")?,
        base: read_f32(view, "base")?,
        multiplier: read_f32(view, "multiplier")?,
    })
}

pub(crate) fn costmap_filter_info_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::CostmapFilterInfo,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_u32(view, "type", bus.r#type)?;
    write_string(view, "filter_mask_topic", &bus.filter_mask_topic)?;
    write_f32(view, "base", bus.base)?;
    write_f32(view, "multiplier", bus.multiplier)?;
    Ok(())
}

pub(crate) fn costmap_filter_info_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::CostmapFilterInfo> {
    costmap_filter_info_from_view(&msg.view())
}

pub(crate) fn costmap_filter_info_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::CostmapFilterInfo,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/CostmapFilterInfo")?;
    costmap_filter_info_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsCostmapFilterInfoMapper;
impl TopicMapper for Nav2MsgsCostmapFilterInfoMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/CostmapFilterInfo"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(costmap_filter_info_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::CostmapFilterInfo as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode nav2_msgs/msg/CostmapFilterInfo: {e}"))
        })?;
        costmap_filter_info_bus_to_dyn(&bus)
    }
}
