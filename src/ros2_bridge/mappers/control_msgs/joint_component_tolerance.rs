//! Typed mapper for `control_msgs/msg/JointComponentTolerance`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_component_tolerance_to_bus(msg: ros_env::control_msgs::msg::JointComponentTolerance) -> crate::control_msgs::msg::v1::JointComponentTolerance {
    crate::control_msgs::msg::v1::JointComponentTolerance {
        joint_name: crate::ros2_bridge::mappers::convert::from_ros_string(msg.joint_name),
        component: msg.component,
        value: msg.value,
    }
}

pub(crate) fn joint_component_tolerance_to_ros(bus: crate::control_msgs::msg::v1::JointComponentTolerance) -> ros_env::control_msgs::msg::JointComponentTolerance {
    ros_env::control_msgs::msg::JointComponentTolerance {
        joint_name: crate::ros2_bridge::mappers::convert::to_ros_string(bus.joint_name),
        component: bus.component,
        value: bus.value,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsJointComponentToleranceMapper;

impl TypedTopicMapper for ControlMsgsJointComponentToleranceMapper {
    type Ros = ros_env::control_msgs::msg::JointComponentTolerance;
    type Bus = crate::control_msgs::msg::v1::JointComponentTolerance;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_component_tolerance_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_component_tolerance_to_ros(msg))
    }
}
