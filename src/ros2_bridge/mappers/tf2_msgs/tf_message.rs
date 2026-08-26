//! Mapper for `tf2_msgs/msg/TFMessage`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn tf_message_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::tf2_msgs::msg::v1::TfMessage> {
    Ok(crate::tf2_msgs::msg::v1::TfMessage {
        transforms: read_message_seq(
            view,
            "transforms",
            super::super::geometry_msgs::transform_stamped::transform_stamped_from_view,
        )?,
    })
}

pub(crate) fn tf_message_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::tf2_msgs::msg::v1::TfMessage,
) -> Result<()> {
    write_message_seq(
        view,
        "transforms",
        &bus.transforms,
        super::super::geometry_msgs::transform_stamped::transform_stamped_write,
    )?;
    Ok(())
}

pub(crate) fn tf_message_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::tf2_msgs::msg::v1::TfMessage> {
    tf_message_from_view(&msg.view())
}

pub(crate) fn tf_message_bus_to_dyn(
    bus: &crate::tf2_msgs::msg::v1::TfMessage,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("tf2_msgs/msg/TFMessage")?;
    tf_message_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Tf2MsgsTfMessageMapper;
impl TopicMapper for Tf2MsgsTfMessageMapper {
    fn type_name(&self) -> &'static str {
        "tf2_msgs/msg/TFMessage"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(tf_message_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::tf2_msgs::msg::v1::TfMessage as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode tf2_msgs/msg/TFMessage: {e}")))?;
        tf_message_bus_to_dyn(&bus)
    }
}
