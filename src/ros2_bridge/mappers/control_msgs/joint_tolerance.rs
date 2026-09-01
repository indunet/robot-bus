//! Typed mapper for `control_msgs/msg/JointTolerance`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_tolerance_to_bus(msg: ros_env::control_msgs::msg::JointTolerance) -> crate::control_msgs::msg::v1::JointTolerance {
    crate::control_msgs::msg::v1::JointTolerance {
        name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.name),
        position: msg.position,
        velocity: msg.velocity,
        acceleration: msg.acceleration,
    }
}

pub(crate) fn joint_tolerance_to_ros(bus: crate::control_msgs::msg::v1::JointTolerance) -> ros_env::control_msgs::msg::JointTolerance {
    ros_env::control_msgs::msg::JointTolerance {
        name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.name),
        position: bus.position,
        velocity: bus.velocity,
        acceleration: bus.acceleration,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsJointToleranceMapper;

impl TypedTopicMapper for ControlMsgsJointToleranceMapper {
    type Ros = ros_env::control_msgs::msg::JointTolerance;
    type Bus = crate::control_msgs::msg::v1::JointTolerance;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_tolerance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_tolerance_to_ros(msg))
    }
}
