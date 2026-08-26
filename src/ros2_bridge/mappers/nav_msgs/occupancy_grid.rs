//! Mapper for `nav_msgs/msg/OccupancyGrid`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn occupancy_grid_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::OccupancyGrid> {
    Ok(crate::nav_msgs::msg::v1::OccupancyGrid {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        info: nested_view(view, "info")?
            .as_ref()
            .map(super::map_meta_data::map_meta_data_from_view)
            .transpose()?,
        data: read_byte_seq(view, "data")?,
    })
}

pub(crate) fn occupancy_grid_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::OccupancyGrid,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.info {
        with_nested_mut(view, "info", |nested| super::map_meta_data::map_meta_data_write(nested, v))?;
    }
    write_byte_seq(view, "data", &bus.data)?;
    Ok(())
}

pub(crate) fn occupancy_grid_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::OccupancyGrid> {
    occupancy_grid_from_view(&msg.view())
}

pub(crate) fn occupancy_grid_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::OccupancyGrid,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/OccupancyGrid")?;
    occupancy_grid_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsOccupancyGridMapper;
impl TopicMapper for NavMsgsOccupancyGridMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/OccupancyGrid"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(occupancy_grid_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::OccupancyGrid as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/OccupancyGrid: {e}")))?;
        occupancy_grid_bus_to_dyn(&bus)
    }
}
