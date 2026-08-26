//! Mapper for `nav2_msgs/msg/SpeedLimit`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn speed_limit_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::SpeedLimit> {
    Ok(crate::nav2_msgs::msg::v1::SpeedLimit {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        percentage: read_bool(view, "percentage")?,
        speed_limit: read_f64(view, "speed_limit")?,
    })
}

pub(crate) fn speed_limit_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::SpeedLimit,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_bool(view, "percentage", bus.percentage)?;
    write_f64(view, "speed_limit", bus.speed_limit)?;
    Ok(())
}

pub(crate) fn speed_limit_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::SpeedLimit> {
    speed_limit_from_view(&msg.view())
}

pub(crate) fn speed_limit_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::SpeedLimit,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/SpeedLimit")?;
    speed_limit_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsSpeedLimitMapper;
impl TopicMapper for Nav2MsgsSpeedLimitMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/SpeedLimit"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(speed_limit_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::SpeedLimit as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/SpeedLimit: {e}")))?;
        speed_limit_bus_to_dyn(&bus)
    }
}
