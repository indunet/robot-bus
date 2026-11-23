//! Typed mapper for `sensor_msgs/msg/JoyFeedback`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn joy_feedback_to_bus(
    msg: ros_env::sensor_msgs::msg::JoyFeedback,
) -> robot_bus::sensor_msgs::msg::v1::JoyFeedback {
    robot_bus::sensor_msgs::msg::v1::JoyFeedback {
        r#type: msg.type_.into(),
        id: msg.id.into(),
        intensity: msg.intensity,
    }
}

pub(crate) fn joy_feedback_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::JoyFeedback,
) -> ros_env::sensor_msgs::msg::JoyFeedback {
    ros_env::sensor_msgs::msg::JoyFeedback {
        type_: bus.r#type as _,
        id: bus.id as _,
        intensity: bus.intensity,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsJoyFeedbackMapper;

impl TypedTopicMapper for SensorMsgsJoyFeedbackMapper {
    type Ros = ros_env::sensor_msgs::msg::JoyFeedback;
    type Bus = robot_bus::sensor_msgs::msg::v1::JoyFeedback;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(joy_feedback_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(joy_feedback_to_ros(msg))
    }
}
