//! Typed mapper for `foxglove_msgs/msg/SceneEntity`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn scene_entity_to_bus(msg: ros_env::foxglove_msgs::msg::SceneEntity) -> crate::foxglove_msgs::msg::v1::SceneEntity {
    crate::foxglove_msgs::msg::v1::SceneEntity {
        timestamp: Some(crate::ros2_bridge::mappers::convert::time_to_timestamp(msg.timestamp)),
        frame_id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.frame_id),
        id: crate::ros2_bridge::mappers::convert::from_ros_string(msg.id),
        lifetime: Some(crate::ros2_bridge::mappers::convert::duration_to_proto(msg.lifetime)),
        frame_locked: msg.frame_locked,
        metadata: msg.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_bus).collect(),
        arrows: msg.arrows.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::arrow_primitive::arrow_primitive_to_bus).collect(),
        cubes: msg.cubes.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::cube_primitive::cube_primitive_to_bus).collect(),
        spheres: msg.spheres.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::sphere_primitive::sphere_primitive_to_bus).collect(),
        cylinders: msg.cylinders.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::cylinder_primitive::cylinder_primitive_to_bus).collect(),
        lines: msg.lines.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::line_primitive::line_primitive_to_bus).collect(),
        triangles: msg.triangles.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::triangle_list_primitive::triangle_list_primitive_to_bus).collect(),
        texts: msg.texts.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::text_primitive::text_primitive_to_bus).collect(),
        models: msg.models.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::model_primitive::model_primitive_to_bus).collect(),
    }
}

pub(crate) fn scene_entity_to_ros(bus: crate::foxglove_msgs::msg::v1::SceneEntity) -> ros_env::foxglove_msgs::msg::SceneEntity {
    ros_env::foxglove_msgs::msg::SceneEntity {
        timestamp: crate::ros2_bridge::mappers::convert::timestamp_to_time(bus.timestamp.unwrap_or_default()),
        frame_id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.frame_id),
        id: crate::ros2_bridge::mappers::convert::to_ros_string(bus.id),
        lifetime: crate::ros2_bridge::mappers::convert::proto_to_duration(bus.lifetime.unwrap_or_default()),
        frame_locked: bus.frame_locked,
        metadata: bus.metadata.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::key_value_pair::key_value_pair_to_ros).collect(),
        arrows: bus.arrows.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::arrow_primitive::arrow_primitive_to_ros).collect(),
        cubes: bus.cubes.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::cube_primitive::cube_primitive_to_ros).collect(),
        spheres: bus.spheres.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::sphere_primitive::sphere_primitive_to_ros).collect(),
        cylinders: bus.cylinders.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::cylinder_primitive::cylinder_primitive_to_ros).collect(),
        lines: bus.lines.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::line_primitive::line_primitive_to_ros).collect(),
        triangles: bus.triangles.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::triangle_list_primitive::triangle_list_primitive_to_ros).collect(),
        texts: bus.texts.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::text_primitive::text_primitive_to_ros).collect(),
        models: bus.models.into_iter().map(crate::ros2_bridge::mappers::foxglove_msgs::model_primitive::model_primitive_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FoxgloveMsgsSceneEntityMapper;

impl TypedTopicMapper for FoxgloveMsgsSceneEntityMapper {
    type Ros = ros_env::foxglove_msgs::msg::SceneEntity;
    type Bus = crate::foxglove_msgs::msg::v1::SceneEntity;

    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/SceneEntity"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(scene_entity_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(scene_entity_to_ros(msg))
    }
}
