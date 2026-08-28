//! Typed mapper for `control_msgs/msg/MultiDOFStateStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn multi_dof_state_stamped_to_bus(msg: ros_env::control_msgs::msg::MultiDOFStateStamped) -> crate::control_msgs::msg::v1::MultiDofStateStamped {
    crate::control_msgs::msg::v1::MultiDofStateStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        dof_states: msg.dof_states.into_iter().map(crate::ros2_bridge::mappers::control_msgs::single_dof_state::single_dof_state_to_bus).collect(),
    }
}

pub(crate) fn multi_dof_state_stamped_to_ros(bus: crate::control_msgs::msg::v1::MultiDofStateStamped) -> ros_env::control_msgs::msg::MultiDOFStateStamped {
    ros_env::control_msgs::msg::MultiDOFStateStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        dof_states: bus.dof_states.into_iter().map(crate::ros2_bridge::mappers::control_msgs::single_dof_state::single_dof_state_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsMultiDofStateStampedMapper;

impl TypedTopicMapper for ControlMsgsMultiDofStateStampedMapper {
    type Ros = ros_env::control_msgs::msg::MultiDOFStateStamped;
    type Bus = crate::control_msgs::msg::v1::MultiDofStateStamped;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MultiDOFStateStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(multi_dof_state_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(multi_dof_state_stamped_to_ros(msg))
    }
}
