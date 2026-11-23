//! Typed mapper for `sensor_msgs/msg/JointState`.

use crate::mapper::TypedTopicMapper;

pub(crate) fn joint_state_to_bus(
    msg: ros_env::sensor_msgs::msg::JointState,
) -> robot_bus::sensor_msgs::msg::v1::JointState {
    robot_bus::sensor_msgs::msg::v1::JointState {
        header: Some(crate::mappers::std_msgs::header::header_to_bus(msg.header)),
        name: crate::mappers::convert::string_seq(msg.name),
        position: crate::mappers::convert::f64_seq(msg.position),
        velocity: crate::mappers::convert::f64_seq(msg.velocity),
        effort: crate::mappers::convert::f64_seq(msg.effort),
    }
}

pub(crate) fn joint_state_to_ros(
    bus: robot_bus::sensor_msgs::msg::v1::JointState,
) -> ros_env::sensor_msgs::msg::JointState {
    ros_env::sensor_msgs::msg::JointState {
        header: crate::mappers::std_msgs::header::header_to_ros(
            bus.header.unwrap_or_default(),
        ),
        name: crate::mappers::convert::ros_string_seq(bus.name),
        position: crate::mappers::convert::FromF64Seq::from_f64_seq(bus.position),
        velocity: crate::mappers::convert::FromF64Seq::from_f64_seq(bus.velocity),
        effort: crate::mappers::convert::FromF64Seq::from_f64_seq(bus.effort),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsJointStateMapper;

impl TypedTopicMapper for SensorMsgsJointStateMapper {
    type Ros = ros_env::sensor_msgs::msg::JointState;
    type Bus = robot_bus::sensor_msgs::msg::v1::JointState;

    fn ros_to_bus(&self, msg: Self::Ros) -> robot_bus::errors::Result<Self::Bus> {
        Ok(joint_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> robot_bus::errors::Result<Self::Ros> {
        Ok(joint_state_to_ros(msg))
    }
}
