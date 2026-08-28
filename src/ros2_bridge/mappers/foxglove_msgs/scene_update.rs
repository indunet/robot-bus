//! Typed mapper for `foxglove_msgs/msg/SceneUpdate`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn scene_update_to_bus(msg: ros_env::foxglove_msgs::msg::SceneUpdate) -> crate::foxglove_msgs::msg::v1::SceneUpdate {
    crate::foxglove_msgs::msg::v1::SceneUpdate {
        deletions: msg.deletions.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::scene_entity_deletion::scene_entity_deletion_to_bus).collect(),
        entities: msg.entities.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::scene_entity::scene_entity_to_bus).collect(),
    }
}

pub(crate) fn scene_update_to_ros(bus: crate::foxglove_msgs::msg::v1::SceneUpdate) -> ros_env::foxglove_msgs::msg::SceneUpdate {
    ros_env::foxglove_msgs::msg::SceneUpdate {
        deletions: bus.deletions.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::scene_entity_deletion::scene_entity_deletion_to_ros).collect(),
        entities: bus.entities.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::scene_entity::scene_entity_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsSceneUpdateMapper;

impl TypedTopicMapper for FoxgloveMsgsSceneUpdateMapper {
    type Ros = ros_env::foxglove_msgs::msg::SceneUpdate;
    type Bus = crate::foxglove_msgs::msg::v1::SceneUpdate;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/SceneUpdate"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(scene_update_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(scene_update_to_ros(msg))
    }
}
