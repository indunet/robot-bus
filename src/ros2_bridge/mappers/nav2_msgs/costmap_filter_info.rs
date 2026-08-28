//! Typed mapper for `nav2_msgs/msg/CostmapFilterInfo`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn costmap_filter_info_to_bus(msg: ros_env::nav2_msgs::msg::CostmapFilterInfo) -> crate::nav2_msgs::msg::v1::CostmapFilterInfo {
    crate::nav2_msgs::msg::v1::CostmapFilterInfo {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        r#type: msg.type_,
        filter_mask_topic: crate::ros2_bridge::mappers::convert::from_ros_string(msg.filter_mask_topic),
        base: msg.base,
        multiplier: msg.multiplier,
    }
}

pub(crate) fn costmap_filter_info_to_ros(bus: crate::nav2_msgs::msg::v1::CostmapFilterInfo) -> ros_env::nav2_msgs::msg::CostmapFilterInfo {
    ros_env::nav2_msgs::msg::CostmapFilterInfo {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        type_: bus.r#type,
        filter_mask_topic: crate::ros2_bridge::mappers::convert::to_ros_string(bus.filter_mask_topic),
        base: bus.base,
        multiplier: bus.multiplier,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsCostmapFilterInfoMapper;

impl TypedTopicMapper for Nav2MsgsCostmapFilterInfoMapper {
    type Ros = ros_env::nav2_msgs::msg::CostmapFilterInfo;
    type Bus = crate::nav2_msgs::msg::v1::CostmapFilterInfo;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/CostmapFilterInfo"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(costmap_filter_info_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(costmap_filter_info_to_ros(msg))
    }
}
