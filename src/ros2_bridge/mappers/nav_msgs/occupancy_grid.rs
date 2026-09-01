//! Typed mapper for `nav_msgs/msg/OccupancyGrid`.
//!
//! Owned `ros_to_bus` so occupancy `data` (`int8[]` ↔ proto `bytes`) moves
//! instead of element-wise copies.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn occupancy_grid_to_bus(
    msg: ros_env::nav_msgs::msg::OccupancyGrid,
) -> crate::nav_msgs::msg::v1::OccupancyGrid {
    crate::nav_msgs::msg::v1::OccupancyGrid {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        info: Some(
            crate::ros2_bridge::mappers::nav_msgs::map_meta_data::map_meta_data_to_bus(msg.info),
        ),
        data: crate::ros2_bridge::mappers::convert::i8_seq_to_bytes(msg.data),
    }
}

pub(crate) fn occupancy_grid_to_ros(
    bus: crate::nav_msgs::msg::v1::OccupancyGrid,
) -> ros_env::nav_msgs::msg::OccupancyGrid {
    ros_env::nav_msgs::msg::OccupancyGrid {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        info: crate::ros2_bridge::mappers::nav_msgs::map_meta_data::map_meta_data_to_ros(
            bus.info.unwrap_or_default(),
        ),
        data: crate::ros2_bridge::mappers::convert::bytes_to_i8_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NavMsgsOccupancyGridMapper;

impl TypedTopicMapper for NavMsgsOccupancyGridMapper {
    type Ros = ros_env::nav_msgs::msg::OccupancyGrid;
    type Bus = crate::nav_msgs::msg::v1::OccupancyGrid;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(occupancy_grid_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
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
}
