//! Mapper for `control_msgs/msg/MultiDOFStateStamped`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn multi_dof_state_stamped_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::MultiDofStateStamped> {
    Ok(crate::control_msgs::msg::v1::MultiDofStateStamped {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        dof_states: read_message_seq(
            view,
            "dof_states",
            super::single_dof_state::single_dof_state_from_view,
        )?,
    })
}

pub(crate) fn multi_dof_state_stamped_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::MultiDofStateStamped,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "dof_states",
        &bus.dof_states,
        super::single_dof_state::single_dof_state_write,
    )?;
    Ok(())
}

pub(crate) fn multi_dof_state_stamped_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::MultiDofStateStamped> {
    multi_dof_state_stamped_from_view(&msg.view())
}

pub(crate) fn multi_dof_state_stamped_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::MultiDofStateStamped,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/MultiDOFStateStamped")?;
    multi_dof_state_stamped_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsMultiDofStateStampedMapper;
impl TopicMapper for ControlMsgsMultiDofStateStampedMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MultiDOFStateStamped"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(multi_dof_state_stamped_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus =
            <crate::control_msgs::msg::v1::MultiDofStateStamped as ProstMessage>::decode(payload)
                .map_err(|e| {
                BusError::Protocol(format!("decode control_msgs/msg/MultiDOFStateStamped: {e}"))
            })?;
        multi_dof_state_stamped_bus_to_dyn(&bus)
    }
}
