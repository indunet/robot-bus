//! Typed mapper for `sensor_msgs/msg/JoyFeedbackArray`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joy_feedback_array_to_bus(
    msg: ros_env::sensor_msgs::msg::JoyFeedbackArray,
) -> crate::sensor_msgs::msg::v1::JoyFeedbackArray {
    crate::sensor_msgs::msg::v1::JoyFeedbackArray {
        array: msg
            .array
            .into_iter()
            .map(crate::ros2_bridge::mappers::sensor_msgs::joy_feedback::joy_feedback_to_bus)
            .collect(),
    }
}

pub(crate) fn joy_feedback_array_to_ros(
    bus: crate::sensor_msgs::msg::v1::JoyFeedbackArray,
) -> ros_env::sensor_msgs::msg::JoyFeedbackArray {
    ros_env::sensor_msgs::msg::JoyFeedbackArray {
        array: bus
            .array
            .into_iter()
            .map(crate::ros2_bridge::mappers::sensor_msgs::joy_feedback::joy_feedback_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsJoyFeedbackArrayMapper;

impl TypedTopicMapper for SensorMsgsJoyFeedbackArrayMapper {
    type Ros = ros_env::sensor_msgs::msg::JoyFeedbackArray;
    type Bus = crate::sensor_msgs::msg::v1::JoyFeedbackArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joy_feedback_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joy_feedback_array_to_ros(msg))
    }
}
