//! Mapper for `nav2_msgs/msg/Particle`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::ros2_bridge::mapper::TopicMapper;
use crate::BusError;


pub(crate) fn particle_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::Particle> {
    Ok(crate::nav2_msgs::msg::v1::Particle {
        pose: nested_view(view, "pose")?
            .as_ref()
            .map(super::super::geometry_msgs::pose::pose_from_view)
            .transpose()?,
        weight: read_f64(view, "weight")?,
    })
}

pub(crate) fn particle_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::Particle,
) -> Result<()> {
    if let Some(v) = &bus.pose {
        with_nested_mut(view, "pose", |nested| {
            super::super::geometry_msgs::pose::pose_write(nested, v)
        })?;
    }
    write_f64(view, "weight", bus.weight)?;
    Ok(())
}

pub(crate) fn particle_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::Particle> {
    particle_from_view(&msg.view())
}

pub(crate) fn particle_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::Particle,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/Particle")?;
    particle_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsParticleMapper;
impl TopicMapper for Nav2MsgsParticleMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/Particle"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(particle_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::Particle as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/Particle: {e}")))?;
        particle_bus_to_dyn(&bus)
    }
}
