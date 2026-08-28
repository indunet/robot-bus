//! Typed mapper for `control_msgs/msg/SingleDOFStateStamped`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn single_dof_state_stamped_to_bus(msg: ros_env::control_msgs::msg::SingleDOFStateStamped) -> crate::control_msgs::msg::v1::SingleDofStateStamped {
    crate::control_msgs::msg::v1::SingleDofStateStamped {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        state: Some(crate::ros2_bridge::mappers::control_msgs::single_dof_state::single_dof_state_to_bus(msg.state)),
    }
}

pub(crate) fn single_dof_state_stamped_to_ros(bus: crate::control_msgs::msg::v1::SingleDofStateStamped) -> ros_env::control_msgs::msg::SingleDOFStateStamped {
    ros_env::control_msgs::msg::SingleDOFStateStamped {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        state: crate::ros2_bridge::mappers::control_msgs::single_dof_state::single_dof_state_to_ros(bus.state.unwrap_or_default()),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControlMsgsSingleDofStateStampedMapper;

impl TypedTopicMapper for ControlMsgsSingleDofStateStampedMapper {
    type Ros = ros_env::control_msgs::msg::SingleDOFStateStamped;
    type Bus = crate::control_msgs::msg::v1::SingleDofStateStamped;

    fn type_name(&self) -> &'static str {
        "control_msgs/msg/SingleDOFStateStamped"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(single_dof_state_stamped_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(single_dof_state_stamped_to_ros(msg))
    }
}
