//! Mapper for `control_msgs/msg/SingleDOFStateStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn single_dof_state_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::SingleDofStateStamped> {
    Ok(crate::control_msgs::msg::v1::SingleDofStateStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        state: nested_view(view, "state")?
            .as_ref()
            .map(super::single_dof_state::single_dof_state_from_view)
            .transpose()?,
    })
}

pub(crate) fn single_dof_state_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::SingleDofStateStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    if let Some(v) = &bus.state {
        with_nested_mut(view, "state", |nested| super::single_dof_state::single_dof_state_write(nested, v))?;
    }
    Ok(())
}

pub(crate) fn single_dof_state_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::SingleDofStateStamped> {
    single_dof_state_stamped_from_view(&msg.view())
}

pub(crate) fn single_dof_state_stamped_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::SingleDofStateStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/SingleDOFStateStamped")?;
    single_dof_state_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsSingleDofStateStampedMapper;
impl TopicMapper for ControlMsgsSingleDofStateStampedMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/SingleDOFStateStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(single_dof_state_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::control_msgs::msg::v1::SingleDofStateStamped as ProstMessage>::decode(payload)
                .map_err(|e| {
                    BusError::Protocol(format!(
                        "decode control_msgs/msg/SingleDOFStateStamped: {e}"
                    ))
                })?;
        single_dof_state_stamped_bus_to_dyn(&bus)
    }
}
