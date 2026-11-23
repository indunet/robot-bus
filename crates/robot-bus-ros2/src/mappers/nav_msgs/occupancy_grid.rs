//! Typed mapper for `nav_msgs/msg/OccupancyGrid`.
//!
//! Owned `ros_to_bus` so occupancy `data` (`int8[]` ↔ proto `bytes`) moves
//! instead of element-wise copies.

use crate::mapper::TypedTopicMapper;

pub(crate) fn occupancy_grid_to_bus(
    msg: ros_env::nav_msgs::msg::OccupancyGrid,
) -> robot_bus::nav_msgs::msg::v1::OccupancyGrid {
    robot_bus::nav_msgs::msg::v1::OccupancyGrid {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        info: Some(
            crate::mappers::nav_msgs::map_meta_data::map_meta_data_to_bus(msg.info),
        ),
        data: crate::mappers::convert::i8_seq_to_bytes(msg.data),
    }
}

pub(crate) fn occupancy_grid_to_ros(
    bus: robot_bus::nav_msgs::msg::v1::OccupancyGrid,
) -> ros_env::nav_msgs::msg::OccupancyGrid {
    ros_env::nav_msgs::msg::OccupancyGrid {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        info: crate::mappers::nav_msgs::map_meta_data::map_meta_data_to_ros(
            bus.info.unwrap_or_default(),
        ),
        data: crate::mappers::convert::bytes_to_i8_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsOccupancyGridMapper;

impl TypedTopicMapper for NavMsgsOccupancyGridMapper {
    type Ros = ros_env::nav_msgs::msg::OccupancyGrid;
    type Bus = robot_bus::nav_msgs::msg::v1::OccupancyGrid;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(occupancy_grid_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(occupancy_grid_to_ros(msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occupancy_grid_i8_bytes_roundtrip() {
        let ros = ros_env::nav_msgs::msg::OccupancyGrid {
            header: Default::default(),
            info: Default::default(),
            data: vec![0, 1, -1, 100],
        };
        let bus = occupancy_grid_to_bus(ros);
        assert_eq!(bus.data, vec![0, 1, 255, 100]);
        let back = occupancy_grid_to_ros(bus);
        assert_eq!(back.data, vec![0, 1, -1, 100]);
    }

    #[test]
    fn occupancy_grid_bulk_i8_roundtrip() {
        let data: Vec<i8> = (0..4096).map(|i| ((i % 251) as i16 - 125) as i8).collect();
        let ros = ros_env::nav_msgs::msg::OccupancyGrid {
            header: Default::default(),
            info: Default::default(),
            data: data.clone(),
        };
        let bus = occupancy_grid_to_bus(ros);
        assert_eq!(bus.data.len(), 4096);
        assert_eq!(occupancy_grid_to_ros(bus).data, data);
    }
}
