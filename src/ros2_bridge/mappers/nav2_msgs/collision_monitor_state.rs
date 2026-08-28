//! Typed mapper for `nav2_msgs/msg/CollisionMonitorState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn collision_monitor_state_to_bus(msg: ros_env::nav2_msgs::msg::CollisionMonitorState) -> crate::nav2_msgs::msg::v1::CollisionMonitorState {
    crate::nav2_msgs::msg::v1::CollisionMonitorState {
        action_type: msg.action_type,
        polygon_name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.polygon_name),
    }
}

pub(crate) fn collision_monitor_state_to_ros(bus: crate::nav2_msgs::msg::v1::CollisionMonitorState) -> ros_env::nav2_msgs::msg::CollisionMonitorState {
    ros_env::nav2_msgs::msg::CollisionMonitorState {
        action_type: bus.action_type,
        polygon_name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.polygon_name),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsCollisionMonitorStateMapper;

impl TypedTopicMapper for Nav2MsgsCollisionMonitorStateMapper {
    type Ros = ros_env::nav2_msgs::msg::CollisionMonitorState;
    type Bus = crate::nav2_msgs::msg::v1::CollisionMonitorState;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/CollisionMonitorState"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(collision_monitor_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(collision_monitor_state_to_ros(msg))
    }
}
