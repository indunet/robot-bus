//! Mapper for `control_msgs/msg/InterfaceValue`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn interface_value_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::InterfaceValue> {
    Ok(crate::control_msgs::msg::v1::InterfaceValue {
        interface_names: read_string_seq(view, "interface_names")?,
        values: read_f64_seq(view, "values")?,
    })
}

pub(crate) fn interface_value_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::InterfaceValue,
) -> Result<()> {
    write_string_seq(view, "interface_names", &bus.interface_names)?;
    write_f64_seq(view, "values", &bus.values)?;
    Ok(())
}

pub(crate) fn interface_value_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::InterfaceValue> {
    interface_value_from_view(&msg.view())
}

pub(crate) fn interface_value_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::InterfaceValue,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/InterfaceValue")?;
    interface_value_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsInterfaceValueMapper;
impl TopicMapper for ControlMsgsInterfaceValueMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/InterfaceValue"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(interface_value_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::InterfaceValue as ProstMessage>::decode(payload)
            .map_err(|e| {
            BusError::Protocol(format!("decode control_msgs/msg/InterfaceValue: {e}"))
        })?;
        interface_value_bus_to_dyn(&bus)
    }
}
