//! Typed mapper for `control_msgs/msg/MotionArgument`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn motion_argument_to_bus(msg: ros_env::control_msgs::msg::MotionArgument) -> crate::control_msgs::msg::v1::MotionArgument {
    crate::control_msgs::msg::v1::MotionArgument {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        value: msg.value,
    }
}

pub(crate) fn motion_argument_to_ros(bus: crate::control_msgs::msg::v1::MotionArgument) -> ros_env::control_msgs::msg::MotionArgument {
    ros_env::control_msgs::msg::MotionArgument {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        value: bus.value,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsMotionArgumentMapper;

impl TypedTopicMapper for ControlMsgsMotionArgumentMapper {
    type Ros = ros_env::control_msgs::msg::MotionArgument;
    type Bus = crate::control_msgs::msg::v1::MotionArgument;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MotionArgument"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(motion_argument_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(motion_argument_to_ros(msg))
    }
}
