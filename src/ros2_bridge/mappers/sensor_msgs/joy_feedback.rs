//! Typed mapper for `sensor_msgs/msg/JoyFeedback`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joy_feedback_to_bus(msg: ros_env::sensor_msgs::msg::JoyFeedback) -> crate::sensor_msgs::msg::v1::JoyFeedback {
    crate::sensor_msgs::msg::v1::JoyFeedback {
        r#type: msg.type_,
        id: msg.id,
        intensity: msg.intensity,
    }
}

pub(crate) fn joy_feedback_to_ros(bus: crate::sensor_msgs::msg::v1::JoyFeedback) -> ros_env::sensor_msgs::msg::JoyFeedback {
    ros_env::sensor_msgs::msg::JoyFeedback {
        type_: bus.r#type,
        id: bus.id,
        intensity: bus.intensity,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsJoyFeedbackMapper;

impl TypedTopicMapper for SensorMsgsJoyFeedbackMapper {
    type Ros = ros_env::sensor_msgs::msg::JoyFeedback;
    type Bus = crate::sensor_msgs::msg::v1::JoyFeedback;

    fn type_name(&self) -> &'static str {
        "sensor_msgs/msg/JoyFeedback"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joy_feedback_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joy_feedback_to_ros(msg))
    }
}
