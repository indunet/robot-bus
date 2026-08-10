//! Mapper for `nav2_msgs/msg/Costmap`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn costmap_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::Costmap> {
    Ok(crate::nav2_msgs::msg::v1::Costmap {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        metadata: nested_view(view, "metadata")?
            .as_ref()
            .map(super::costmap_meta_data::costmap_meta_data_from_view)
            .transpose()?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn costmap_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::Costmap,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.metadata {
        with_nested_mut(view, "metadata", |nested| {
            super::costmap_meta_data::costmap_meta_data_write(nested, v)
        })?;
    }
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn costmap_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::Costmap> {
    costmap_from_view(&msg.view())
}

pub(crate) fn costmap_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::Costmap,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/Costmap")?;
    costmap_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsCostmapMapper;
impl TopicMapper for Nav2MsgsCostmapMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/Costmap"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(costmap_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::Costmap as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/Costmap: {e}")))?;
        costmap_bus_to_dyn(&bus)
    }
}
