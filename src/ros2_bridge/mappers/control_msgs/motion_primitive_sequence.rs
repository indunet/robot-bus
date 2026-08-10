//! Mapper for `control_msgs/msg/MotionPrimitiveSequence`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn motion_primitive_sequence_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::MotionPrimitiveSequence> {
    Ok(crate::control_msgs::msg::v1::MotionPrimitiveSequence {
        motions: read_message_seq(view, "motions", super::motion_primitive::motion_primitive_from_view)?,
    })
}

pub(crate) fn motion_primitive_sequence_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::MotionPrimitiveSequence,
) -> Result<()> {
    write_message_seq(view, "motions", &bus.motions, super::motion_primitive::motion_primitive_write)?;
    Ok(())
}

pub(crate) fn motion_primitive_sequence_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::MotionPrimitiveSequence> {
    motion_primitive_sequence_from_view(&msg.view())
}

pub(crate) fn motion_primitive_sequence_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::MotionPrimitiveSequence,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/MotionPrimitiveSequence")?;
    motion_primitive_sequence_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsMotionPrimitiveSequenceMapper;
impl TopicMapper for ControlMsgsMotionPrimitiveSequenceMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MotionPrimitiveSequence"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(motion_primitive_sequence_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::MotionPrimitiveSequence as ProstMessage>::decode(
            payload,
        )
        .map_err(|e| {
            BusError::Protocol(format!(
                "decode control_msgs/msg/MotionPrimitiveSequence: {e}"
            ))
        })?;
        motion_primitive_sequence_bus_to_dyn(&bus)
    }
}
