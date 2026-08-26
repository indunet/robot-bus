//! Mapper for `foxglove_msgs/msg/SceneEntityDeletion`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn scene_entity_deletion_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::foxglove_msgs::msg::v1::SceneEntityDeletion> {
    Ok(crate::foxglove_msgs::msg::v1::SceneEntityDeletion {
        timestamp: read_timestamp(view, "timestamp")?,
        r#type: read_i32(view, "type")?,
        id: read_string(view, "id")?,
    })
}

pub(crate) fn scene_entity_deletion_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::foxglove_msgs::msg::v1::SceneEntityDeletion,
) -> Result<()> {
    if let Some(v) = &bus.timestamp {
        write_timestamp(view, "timestamp", v)?;
    }
    write_i32(view, "type", bus.r#type)?;
    write_string(view, "id", &bus.id)?;
    Ok(())
}

pub(crate) fn scene_entity_deletion_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::foxglove_msgs::msg::v1::SceneEntityDeletion> {
    scene_entity_deletion_from_view(&msg.view())
}

pub(crate) fn scene_entity_deletion_bus_to_dyn(
    bus: &crate::foxglove_msgs::msg::v1::SceneEntityDeletion,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("foxglove_msgs/msg/SceneEntityDeletion")?;
    scene_entity_deletion_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct FoxgloveMsgsSceneEntityDeletionMapper;
impl TopicMapper for FoxgloveMsgsSceneEntityDeletionMapper {
    fn type_name(&self) -> &'static str {
        "foxglove_msgs/msg/SceneEntityDeletion"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(scene_entity_deletion_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::foxglove_msgs::msg::v1::SceneEntityDeletion as ProstMessage>::decode(payload)
                .map_err(|e| {
                BusError::Protocol(format!("decode foxglove_msgs/msg/SceneEntityDeletion: {e}"))
            })?;
        scene_entity_deletion_bus_to_dyn(&bus)
    }
}
