//! Mapper for `control_msgs/msg/MotionPrimitive`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn motion_primitive_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::control_msgs::msg::v1::MotionPrimitive> {
    Ok(crate::control_msgs::msg::v1::MotionPrimitive {
        r#type: read_i32(view, "type")?,
        blend_radius: read_f64(view, "blend_radius")?,
        additional_arguments: read_message_seq(
            view,
            "additional_arguments",
            super::motion_argument::motion_argument_from_view,
        )?,
        poses: read_message_seq(view, "poses", super::super::geometry_msgs::pose_stamped::pose_stamped_from_view)?,
        joint_positions: read_f64_seq(view, "joint_positions")?,
    })
}

pub(crate) fn motion_primitive_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::control_msgs::msg::v1::MotionPrimitive,
) -> Result<()> {
    write_i32(view, "type", bus.r#type)?;
    write_f64(view, "blend_radius", bus.blend_radius)?;
    write_message_seq(
        view,
        "additional_arguments",
        &bus.additional_arguments,
        super::motion_argument::motion_argument_write,
    )?;
    write_message_seq(
        view,
        "poses",
        &bus.poses,
        super::super::geometry_msgs::pose_stamped::pose_stamped_write,
    )?;
    write_f64_seq(view, "joint_positions", &bus.joint_positions)?;
    Ok(())
}

pub(crate) fn motion_primitive_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::control_msgs::msg::v1::MotionPrimitive> {
    motion_primitive_from_view(&msg.view())
}

pub(crate) fn motion_primitive_bus_to_dyn(
    bus: &crate::control_msgs::msg::v1::MotionPrimitive,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("control_msgs/msg/MotionPrimitive")?;
    motion_primitive_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct ControlMsgsMotionPrimitiveMapper;
impl TopicMapper for ControlMsgsMotionPrimitiveMapper {
    fn type_name(&self) -> &'static str {
        "control_msgs/msg/MotionPrimitive"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(motion_primitive_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::control_msgs::msg::v1::MotionPrimitive as ProstMessage>::decode(payload)
            .map_err(|e| {
                BusError::Protocol(format!("decode control_msgs/msg/MotionPrimitive: {e}"))
            })?;
        motion_primitive_bus_to_dyn(&bus)
    }
}
