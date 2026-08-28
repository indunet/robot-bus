//! Typed mapper for `nav2_msgs/msg/SpeedLimit`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn speed_limit_to_bus(msg: ros_env::nav2_msgs::msg::SpeedLimit) -> crate::nav2_msgs::msg::v1::SpeedLimit {
    crate::nav2_msgs::msg::v1::SpeedLimit {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        percentage: msg.percentage,
        speed_limit: msg.speed_limit,
    }
}

pub(crate) fn speed_limit_to_ros(bus: crate::nav2_msgs::msg::v1::SpeedLimit) -> ros_env::nav2_msgs::msg::SpeedLimit {
    ros_env::nav2_msgs::msg::SpeedLimit {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        percentage: bus.percentage,
        speed_limit: bus.speed_limit,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsSpeedLimitMapper;

impl TypedTopicMapper for Nav2MsgsSpeedLimitMapper {
    type Ros = ros_env::nav2_msgs::msg::SpeedLimit;
    type Bus = crate::nav2_msgs::msg::v1::SpeedLimit;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/SpeedLimit"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(speed_limit_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(speed_limit_to_ros(msg))
    }
}
