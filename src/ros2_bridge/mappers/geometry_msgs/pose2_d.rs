//! Mapper for `geometry_msgs/msg/Pose2D`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn pose2_d_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::geometry_msgs::msg::v1::Pose2D> {
    Ok(crate::geometry_msgs::msg::v1::Pose2D {
        x: read_f64(view, "x")?,
        y: read_f64(view, "y")?,
        theta: read_f64(view, "theta")?,
    })
}

pub(crate) fn pose2_d_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::geometry_msgs::msg::v1::Pose2D,
) -> Result<()> {
    write_f64(view, "x", bus.x)?;
    write_f64(view, "y", bus.y)?;
    write_f64(view, "theta", bus.theta)?;
    Ok(())
}

pub(crate) fn pose2_d_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::geometry_msgs::msg::v1::Pose2D> {
    pose2_d_from_view(&msg.view())
}

pub(crate) fn pose2_d_bus_to_dyn(
    bus: &crate::geometry_msgs::msg::v1::Pose2D,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("geometry_msgs/msg/Pose2D")?;
    pose2_d_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct GeometryMsgsPose2DMapper;
impl TopicMapper for GeometryMsgsPose2DMapper {
    fn type_name(&self) -> &'static str {
        "geometry_msgs/msg/Pose2D"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(pose2_d_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::geometry_msgs::msg::v1::Pose2D as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode geometry_msgs/msg/Pose2D: {e}")))?;
        pose2_d_bus_to_dyn(&bus)
    }
}
