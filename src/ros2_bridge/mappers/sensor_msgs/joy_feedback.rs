//! Mapper for `sensor_msgs/msg/JoyFeedback`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn joy_feedback_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::JoyFeedback> {
    Ok(crate::sensor_msgs::msg::v1::JoyFeedback {
        r#type: read_u32(view, "type")?,
        id: read_u32(view, "id")?,
        intensity: read_f32(view, "intensity")?,
    })
}

pub(crate) fn joy_feedback_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::JoyFeedback,
) -> Result<()> {
    write_u32(view, "type", bus.r#type)?;
    write_u32(view, "id", bus.id)?;
    write_f32(view, "intensity", bus.intensity)?;
    Ok(())
}

pub(crate) fn joy_feedback_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::JoyFeedback> {
    joy_feedback_from_view(&msg.view())
}

pub(crate) fn joy_feedback_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::JoyFeedback,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/JoyFeedback")?;
    joy_feedback_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsJoyFeedbackMapper;
impl TopicMapper for SensorMsgsJoyFeedbackMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/JoyFeedback"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joy_feedback_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::JoyFeedback as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode sensor_msgs/msg/JoyFeedback: {e}")))?;
        joy_feedback_bus_to_dyn(&bus)
    }
}
