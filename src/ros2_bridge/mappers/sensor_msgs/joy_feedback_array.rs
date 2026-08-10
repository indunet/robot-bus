//! Mapper for `sensor_msgs/msg/JoyFeedbackArray`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn joy_feedback_array_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::sensor_msgs::msg::v1::JoyFeedbackArray> {
    Ok(crate::sensor_msgs::msg::v1::JoyFeedbackArray {
        array: read_message_seq(view, "array", super::joy_feedback::joy_feedback_from_view)?,
    })
}

pub(crate) fn joy_feedback_array_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::sensor_msgs::msg::v1::JoyFeedbackArray,
) -> Result<()> {
    write_message_seq(view, "array", &bus.array, super::joy_feedback::joy_feedback_write)?;
    Ok(())
}

pub(crate) fn joy_feedback_array_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::sensor_msgs::msg::v1::JoyFeedbackArray> {
    joy_feedback_array_from_view(&msg.view())
}

pub(crate) fn joy_feedback_array_bus_to_dyn(
    bus: &crate::sensor_msgs::msg::v1::JoyFeedbackArray,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("sensor_msgs/msg/JoyFeedbackArray")?;
    joy_feedback_array_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct SensorMsgsJoyFeedbackArrayMapper;
impl TopicMapper for SensorMsgsJoyFeedbackArrayMapper {
    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/JoyFeedbackArray"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(joy_feedback_array_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::sensor_msgs::msg::v1::JoyFeedbackArray as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode sensor_msgs/msg/JoyFeedbackArray: {e}"))
            })?;
        joy_feedback_array_bus_to_dyn(&bus)
    }
}
