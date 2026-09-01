//! Typed mapper for `foxglove_msgs/msg/LocationFixes`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn location_fixes_to_bus(msg: ros_env::foxglove_msgs::msg::LocationFixes) -> crate::foxglove_msgs::msg::v1::LocationFixes {
    crate::foxglove_msgs::msg::v1::LocationFixes {
        fixes: msg.fixes.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::location_fix::location_fix_to_bus).collect(),
    }
}

pub(crate) fn location_fixes_to_ros(bus: crate::foxglove_msgs::msg::v1::LocationFixes) -> ros_env::foxglove_msgs::msg::LocationFixes {
    ros_env::foxglove_msgs::msg::LocationFixes {
        fixes: bus.fixes.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::location_fix::location_fix_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsLocationFixesMapper;

impl TypedTopicMapper for FoxgloveMsgsLocationFixesMapper {
    type Ros = ros_env::foxglove_msgs::msg::LocationFixes;
    type Bus = crate::foxglove_msgs::msg::v1::LocationFixes;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(location_fixes_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(location_fixes_to_ros(msg))
    }
}
