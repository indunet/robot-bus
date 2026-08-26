//! Mapper for `unique_identifier_msgs/msg/UUID`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn uuid_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::unique_identifier_msgs::msg::v1::Uuid> {
    Ok(crate::unique_identifier_msgs::msg::v1::Uuid {
        uuid: read_byte_seq(view, "uuid")?,
    })
}

pub(crate) fn uuid_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::unique_identifier_msgs::msg::v1::Uuid,
) -> Result<()> {
    write_byte_seq(view, "uuid", &bus.uuid)?;
    Ok(())
}

pub(crate) fn uuid_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::unique_identifier_msgs::msg::v1::Uuid> {
    uuid_from_view(&msg.view())
}

pub(crate) fn uuid_bus_to_dyn(
    bus: &crate::unique_identifier_msgs::msg::v1::Uuid,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("unique_identifier_msgs/msg/UUID")?;
    uuid_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct UniqueIdentifierMsgsUuidMapper;
impl TopicMapper for UniqueIdentifierMsgsUuidMapper {
    fn type_name(&self) -> &'static str {
        "unique_identifier_msgs/msg/UUID"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(uuid_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::unique_identifier_msgs::msg::v1::Uuid as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode unique_identifier_msgs/msg/UUID: {e}"))
        })?;
        uuid_bus_to_dyn(&bus)
    }
}
