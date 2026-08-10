//! Mapper for `foxglove_msgs/msg/SceneUpdate`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn scene_update_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::SceneUpdate> {
    Ok(crate::foxglove_msgs::msg::v1::SceneUpdate {
        deletions: read_message_seq(view, "deletions", super::scene_entity_deletion::scene_entity_deletion_from_view)?,
        entities: read_message_seq(view, "entities", super::scene_entity::scene_entity_from_view)?,
    })
}

pub(crate) fn scene_update_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::SceneUpdate,
) -> Result<()> {
    write_message_seq(
        view,
        "deletions",
        &bus.deletions,
        super::scene_entity_deletion::scene_entity_deletion_write,
    )?;
    write_message_seq(view, "entities", &bus.entities, super::scene_entity::scene_entity_write)?;
    Ok(())
}

pub(crate) fn scene_update_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::SceneUpdate> {
    scene_update_from_view(&msg.view())
}

pub(crate) fn scene_update_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::SceneUpdate,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/SceneUpdate")?;
    scene_update_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsSceneUpdateMapper;
impl TopicMapper for FoxgloveMsgsSceneUpdateMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/SceneUpdate"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(scene_update_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::foxglove_msgs::msg::v1::SceneUpdate as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/SceneUpdate: {e}"))
            })?;
        scene_update_bus_to_dyn(&bus)
    }
}
