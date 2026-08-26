//! Mapper for `nav2_msgs/msg/EdgeCost`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn edge_cost_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::EdgeCost> {
    Ok(crate::nav2_msgs::msg::v1::EdgeCost {
        edgeid: read_u32(view, "edgeid")?,
        cost: read_f32(view, "cost")?,
    })
}

pub(crate) fn edge_cost_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::EdgeCost,
) -> Result<()> {
    write_u32(view, "edgeid", bus.edgeid)?;
    write_f32(view, "cost", bus.cost)?;
    Ok(())
}

pub(crate) fn edge_cost_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::EdgeCost> {
    edge_cost_from_view(&msg.view())
}

pub(crate) fn edge_cost_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::EdgeCost,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/EdgeCost")?;
    edge_cost_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsEdgeCostMapper;
impl TopicMapper for Nav2MsgsEdgeCostMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/EdgeCost"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(edge_cost_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::EdgeCost as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/EdgeCost: {e}")))?;
        edge_cost_bus_to_dyn(&bus)
    }
}
