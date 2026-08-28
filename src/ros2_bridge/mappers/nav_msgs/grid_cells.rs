//! Typed mapper for `nav_msgs/msg/GridCells`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn grid_cells_to_bus(msg: ros_env::nav_msgs::msg::GridCells) -> crate::nav_msgs::msg::v1::GridCells {
    crate::nav_msgs::msg::v1::GridCells {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        cell_width: msg.cell_width,
        cell_height: msg.cell_height,
        cells: msg.cells.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_bus).collect(),
    }
}

pub(crate) fn grid_cells_to_ros(bus: crate::nav_msgs::msg::v1::GridCells) -> ros_env::nav_msgs::msg::GridCells {
    ros_env::nav_msgs::msg::GridCells {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        cell_width: bus.cell_width,
        cell_height: bus.cell_height,
        cells: bus.cells.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::point::point_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsGridCellsMapper;

impl TypedTopicMapper for NavMsgsGridCellsMapper {
    type Ros = ros_env::nav_msgs::msg::GridCells;
    type Bus = crate::nav_msgs::msg::v1::GridCells;

    fn type_name(&self) -> &'static str {
        "nav_msgs/msg/GridCells"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(grid_cells_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(grid_cells_to_ros(msg))
    }
}
