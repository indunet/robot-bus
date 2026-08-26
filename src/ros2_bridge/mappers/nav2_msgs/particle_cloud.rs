//! Mapper for `nav2_msgs/msg/ParticleCloud`.

use prost::Message as ProstMessage;
use rclrs::DynamicMessage;

use super::super::common::*;
use crate::BusError;
use crate::ros2_bridge::mapper::TopicMapper;

pub(crate) fn particle_cloud_from_view(
    view: &rclrs::DynamicMessageView<'_>,
) -> Result<crate::nav2_msgs::msg::v1::ParticleCloud> {
    Ok(crate::nav2_msgs::msg::v1::ParticleCloud {
        header: nested_view(view, "header")?
            .as_ref()
            .map(super::super::std_msgs::header::header_from_view)
            .transpose()?,
        particles: read_message_seq(view, "particles", super::particle::particle_from_view)?,
    })
}

pub(crate) fn particle_cloud_write(
    view: &mut rclrs::DynamicMessageViewMut<'_>,
    bus: &crate::nav2_msgs::msg::v1::ParticleCloud,
) -> Result<()> {
    if let Some(v) = &bus.header {
        with_nested_mut(view, "header", |nested| {
            super::super::std_msgs::header::header_write(nested, v)
        })?;
    }
    write_message_seq(
        view,
        "particles",
        &bus.particles,
        super::particle::particle_write,
    )?;
    Ok(())
}

pub(crate) fn particle_cloud_dyn_to_bus(
    msg: &rclrs::DynamicMessage,
) -> Result<crate::nav2_msgs::msg::v1::ParticleCloud> {
    particle_cloud_from_view(&msg.view())
}

pub(crate) fn particle_cloud_bus_to_dyn(
    bus: &crate::nav2_msgs::msg::v1::ParticleCloud,
) -> Result<rclrs::DynamicMessage> {
    let mut msg = new_message("nav2_msgs/msg/ParticleCloud")?;
    particle_cloud_write(&mut msg.view_mut(), bus)?;
    Ok(msg)
}

pub struct Nav2MsgsParticleCloudMapper;
impl TopicMapper for Nav2MsgsParticleCloudMapper {
    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/ParticleCloud"
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>> {
        Ok(particle_cloud_dyn_to_bus(msg)?.encode_to_vec())
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage> {
        let bus = <crate::nav2_msgs::msg::v1::ParticleCloud as ProstMessage>::decode(payload)
            .map_err(|e| BusError::Protocol(format!("decode nav2_msgs/msg/ParticleCloud: {e}")))?;
        particle_cloud_bus_to_dyn(&bus)
    }
}
