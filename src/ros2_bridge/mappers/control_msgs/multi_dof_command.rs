//! Typed mapper for `control_msgs/msg/MultiDOFCommand`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn multi_dof_command_to_bus(msg: ros_env::control_msgs::msg::MultiDOFCommand) -> crate::control_msgs::msg::v1::MultiDofCommand {
    crate::control_msgs::msg::v1::MultiDofCommand {
        dof_names: crate::ros2_bridge::mappers::convert::string_seq(msg.dof_names),
        values: crate::ros2_bridge::mappers::convert::f64_seq(msg.values),
        values_dot: crate::ros2_bridge::mappers::convert::f64_seq(msg.values_dot),
    }
}

pub(crate) fn multi_dof_command_to_ros(bus: crate::control_msgs::msg::v1::MultiDofCommand) -> ros_env::control_msgs::msg::MultiDOFCommand {
    ros_env::control_msgs::msg::MultiDOFCommand {
        dof_names: crate::ros2_bridge::mappers::convert::ros_string_seq(bus.dof_names),
        values: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.values),
        values_dot: crate::ros2_bridge::mappers::convert::FromF64Seq::from_f64_seq(bus.values_dot),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsMultiDofCommandMapper;

impl TypedTopicMapper for ControlMsgsMultiDofCommandMapper {
    type Ros = ros_env::control_msgs::msg::MultiDOFCommand;
    type Bus = crate::control_msgs::msg::v1::MultiDofCommand;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(multi_dof_command_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(multi_dof_command_to_ros(msg))
    }
}
