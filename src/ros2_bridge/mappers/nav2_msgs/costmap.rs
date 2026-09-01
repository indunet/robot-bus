//! Typed mapper for `nav2_msgs/msg/Costmap`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn costmap_to_bus(msg: ros_env::nav2_msgs::msg::Costmap) -> crate::nav2_msgs::msg::v1::Costmap {
    crate::nav2_msgs::msg::v1::Costmap {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        metadata: Some(crate::ros2_bridge::mappers::nav2_msgs::costmap_meta_data::costmap_meta_data_to_bus(msg.metadata)),
        data: crate::ros2_bridge::mappers::convert::i8_seq_to_bytes(msg.data),
    }
}

pub(crate) fn costmap_to_ros(bus: crate::nav2_msgs::msg::v1::Costmap) -> ros_env::nav2_msgs::msg::Costmap {
    ros_env::nav2_msgs::msg::Costmap {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        metadata: crate::ros2_bridge::mappers::nav2_msgs::costmap_meta_data::costmap_meta_data_to_ros(bus.metadata.unwrap_or_default()),
        data: crate::ros2_bridge::mappers::convert::bytes_to_i8_seq(bus.data),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsCostmapMapper;

impl TypedTopicMapper for Nav2MsgsCostmapMapper {
    type Ros = ros_env::nav2_msgs::msg::Costmap;
    type Bus = crate::nav2_msgs::msg::v1::Costmap;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(costmap_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(costmap_to_ros(msg))
    }
}
