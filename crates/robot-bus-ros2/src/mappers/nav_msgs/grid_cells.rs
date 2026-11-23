//! Typed mapper for `nav_msgs/msg/GridCells`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn grid_cells_to_bus(
    msg: ros_env::nav_msgs::msg::GridCells,
) -> robot_bus::nav_msgs::msg::v1::GridCells {
    robot_bus::nav_msgs::msg::v1::GridCells {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        cell_width: msg.cell_width,
        cell_height: msg.cell_height,
        cells: msg
            .cells
            .into_iter()
            .map(crate::mappers::geometry_msgs::point::point_to_bus)
            .collect(),
    }
}

pub(crate) fn grid_cells_to_ros(
    bus: robot_bus::nav_msgs::msg::v1::GridCells,
) -> ros_env::nav_msgs::msg::GridCells {
    ros_env::nav_msgs::msg::GridCells {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        cell_width: bus.cell_width,
        cell_height: bus.cell_height,
        cells: bus
            .cells
            .into_iter()
            .map(crate::mappers::geometry_msgs::point::point_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsGridCellsMapper;

impl TypedTopicMapper for NavMsgsGridCellsMapper {
    type Ros = ros_env::nav_msgs::msg::GridCells;
    type Bus = robot_bus::nav_msgs::msg::v1::GridCells;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(grid_cells_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(grid_cells_to_ros(msg))
    }
}
