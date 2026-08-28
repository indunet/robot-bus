//! Typed mapper for `control_msgs/msg/JointJog`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn joint_jog_to_bus(msg: ros_env::control_msgs::msg::JointJog) -> crate::control_msgs::msg::v1::JointJog {
    crate::control_msgs::msg::v1::JointJog {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        joint_names: crate::ros2_bridge::mappers::convert::string_seq(msg.joint_names),
        displacements: crate::ros2_bridge::mappers::convert::f64_seq(msg.displacements),
        velocities: crate::ros2_bridge::mappers::convert::f64_seq(msg.velocities),
        duration: msg.duration,
    }
}

pub(crate) fn joint_jog_to_ros(bus: crate::control_msgs::msg::v1::JointJog) -> ros_env::control_msgs::msg::JointJog {
    ros_env::control_msgs::msg::JointJog {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        joint_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.joint_names),
        displacements: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.displacements),
        velocities: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.velocities),
        duration: bus.duration,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsJointJogMapper;

impl TypedTopicMapper for ControlMsgsJointJogMapper {
    type Ros = ros_env::control_msgs::msg::JointJog;
    type Bus = crate::control_msgs::msg::v1::JointJog;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/JointJog"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(joint_jog_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(joint_jog_to_ros(msg))
    }
}
