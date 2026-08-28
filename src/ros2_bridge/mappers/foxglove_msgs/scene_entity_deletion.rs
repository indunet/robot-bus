//! Typed mapper for `foxglove_msgs/msg/SceneEntityDeletion`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn scene_entity_deletion_to_bus(msg: ros_env::foxglove_msgs::msg::SceneEntityDeletion) -> crate::foxglove_msgs::msg::v1::SceneEntityDeletion {
    crate::foxglove_msgs::msg::v1::SceneEntityDeletion {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        r#type: msg.type_ as i32,
        id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.id),
    }
}

pub(crate) fn scene_entity_deletion_to_ros(bus: crate::foxglove_msgs::msg::v1::SceneEntityDeletion) -> ros_env::foxglove_msgs::msg::SceneEntityDeletion {
    ros_env::foxglove_msgs::msg::SceneEntityDeletion {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        type_: bus.r#type as i32,
        id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.id),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsSceneEntityDeletionMapper;

impl TypedTopicMapper for FoxgloveMsgsSceneEntityDeletionMapper {
    type Ros = ros_env::foxglove_msgs::msg::SceneEntityDeletion;
    type Bus = crate::foxglove_msgs::msg::v1::SceneEntityDeletion;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/SceneEntityDeletion"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(scene_entity_deletion_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(scene_entity_deletion_to_ros(msg))
    }
}
