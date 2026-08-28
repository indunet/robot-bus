//! Typed mapper for `control_msgs/msg/MotionPrimitiveSequence`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn motion_primitive_sequence_to_bus(msg: ros_env::control_msgs::msg::MotionPrimitiveSequence) -> crate::control_msgs::msg::v1::MotionPrimitiveSequence {
    crate::control_msgs::msg::v1::MotionPrimitiveSequence {
        motions: msg.motions.into_iter().map(crate::ros2_bridge::mappers::control_msgs::motion_primitive::motion_primitive_to_bus).collect(),
    }
}

pub(crate) fn motion_primitive_sequence_to_ros(bus: crate::control_msgs::msg::v1::MotionPrimitiveSequence) -> ros_env::control_msgs::msg::MotionPrimitiveSequence {
    ros_env::control_msgs::msg::MotionPrimitiveSequence {
        motions: bus.motions.into_iter().map(crate::ros2_bridge::mappers::control_msgs::motion_primitive::motion_primitive_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsMotionPrimitiveSequenceMapper;

impl TypedTopicMapper for ControlMsgsMotionPrimitiveSequenceMapper {
    type Ros = ros_env::control_msgs::msg::MotionPrimitiveSequence;
    type Bus = crate::control_msgs::msg::v1::MotionPrimitiveSequence;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MotionPrimitiveSequence"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(motion_primitive_sequence_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(motion_primitive_sequence_to_ros(msg))
    }
}
