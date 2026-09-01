//! Typed mapper for `sensor_msgs/msg/MultiDOFJointState`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn multi_dof_joint_state_to_bus(msg: ros_env::sensor_msgs::msg::MultiDOFJointState) -> crate::sensor_msgs::msg::v1::MultiDofJointState {
    crate::sensor_msgs::msg::v1::MultiDofJointState {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        joint_names: crate::ros2_bridge::mappers::convert::string_seq(msg.joint_names),
        transforms: msg.transforms.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::transform::transform_to_bus).collect(),
        twist: msg.twist.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_bus).collect(),
        wrench: msg.wrench.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::wrench::wrench_to_bus).collect(),
    }
}

pub(crate) fn multi_dof_joint_state_to_ros(bus: crate::sensor_msgs::msg::v1::MultiDofJointState) -> ros_env::sensor_msgs::msg::MultiDOFJointState {
    ros_env::sensor_msgs::msg::MultiDOFJointState {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        joint_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.joint_names),
        transforms: bus.transforms.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::transform::transform_to_ros).collect(),
        twist: bus.twist.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::twist::twist_to_ros).collect(),
        wrench: bus.wrench.into_iter().map(crate::ros2_bridge::mappers::geometry_msgs::wrench::wrench_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SensorMsgsMultiDofJointStateMapper;

impl TypedTopicMapper for SensorMsgsMultiDofJointStateMapper {
    type Ros = ros_env::sensor_msgs::msg::MultiDOFJointState;
    type Bus = crate::sensor_msgs::msg::v1::MultiDofJointState;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(multi_dof_joint_state_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(multi_dof_joint_state_to_ros(msg))
    }
}
