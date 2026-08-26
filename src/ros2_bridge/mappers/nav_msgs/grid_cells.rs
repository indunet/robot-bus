//! Mapper for `nav_msgs/msg/GridCells`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn grid_cells_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav_msgs::msg::v1::GridCells> {
    Ok(crate::nav_msgs::msg::v1::GridCells {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        cell_width: read_f32(view, "cell_width")?,
        cell_height: read_f32(view, "cell_height")?,
        cells: read_message_seq(
            view,
            "cells",
            super::super::geometry_msgs::point::point_from_view,
        )?,
    })
}

pub(crate) fn grid_cells_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav_msgs::msg::v1::GridCells,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_f32(view, "cell_width", bus.cell_width)?;
    write_f32(view, "cell_height", bus.cell_height)?;
    write_message_seq(
        view,
        "cells",
        &bus.cells,
        super::super::geometry_msgs::point::point_write,
    )?;
    Ok(())
}

pub(crate) fn grid_cells_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav_msgs::msg::v1::GridCells> {
    grid_cells_from_view(&msg.view())
}

pub(crate) fn grid_cells_bus_to_dyn(
    bus: &crate::nav_msgs::msg::v1::GridCells,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav_msgs/msg/GridCells")?;
    grid_cells_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct NavMsgsGridCellsMapper;
impl TopicMapper for NavMsgsGridCellsMapper {
    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/GridCells"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(grid_cells_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav_msgs::msg::v1::GridCells as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav_msgs/msg/GridCells: {e}")))?;
        grid_cells_bus_to_dyn(&bus)
    }
}
