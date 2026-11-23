//! Typed mapper for `sensor_msgs/msg/JoyFeedbackArray`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn joy_feedback_array_to_bus(
    msg: ros_env::sensor_msgs::msg::JoyFeedbackArray,
) -> robot_bus::sensor_msgs::msg::v1::JoyFeedbackArray {
    robot_bus::sensor_msgs::msg::v1::JoyFeedbackArray {
        array: msg
            .array
            .into_iter()
            .map(crate::mappers::sensor_msgs::joy_feedback::joy_feedback_to_bus)
            .collect(),
    }
}

pub(crate) fn joy_feedback_array_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::JoyFeedbackArray,
) -> ros_env::sensor_msgs::msg::JoyFeedbackArray {
    ros_env::sensor_msgs::msg::JoyFeedbackArray {
        array: bus
            .array
            .into_iter()
            .map(crate::mappers::sensor_msgs::joy_feedback::joy_feedback_to_ros)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsJoyFeedbackArrayMapper;

impl TypedTopicMapper for SensorMsgsJoyFeedbackArrayMapper {
    type Ros = ros_env::sensor_msgs::msg::JoyFeedbackArray;
    type Bus = robot_bus::sensor_msgs::msg::v1::JoyFeedbackArray;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(joy_feedback_array_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(joy_feedback_array_to_ros(msg))
    }
}
