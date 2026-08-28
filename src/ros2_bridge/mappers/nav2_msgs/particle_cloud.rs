//! Typed mapper for `nav2_msgs/msg/ParticleCloud`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn particle_cloud_to_bus(msg: ros_env::nav2_msgs::msg::ParticleCloud) -> crate::nav2_msgs::msg::v1::ParticleCloud {
    crate::nav2_msgs::msg::v1::ParticleCloud {
        header: Some(crate::ros2_bridge::mappers::std_msgs::header::header_to_bus(msg.header)),
        particles: msg.particles.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::particle::particle_to_bus).collect(),
    }
}

pub(crate) fn particle_cloud_to_ros(bus: crate::nav2_msgs::msg::v1::ParticleCloud) -> ros_env::nav2_msgs::msg::ParticleCloud {
    ros_env::nav2_msgs::msg::ParticleCloud {
        header: crate::ros2_bridge::mappers::std_msgs::header::header_to_ros(bus.header.unwrap_or_default()),
        particles: bus.particles.into_iter().map(crate::ros2_bridge::mappers::nav2_msgs::particle::particle_to_ros).collect(),
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsParticleCloudMapper;

impl TypedTopicMapper for Nav2MsgsParticleCloudMapper {
    type Ros = ros_env::nav2_msgs::msg::ParticleCloud;
    type Bus = crate::nav2_msgs::msg::v1::ParticleCloud;

    fn type_name(&self) -> &'static str {
        "nav2_msgs/msg/ParticleCloud"
    }

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(particle_cloud_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(particle_cloud_to_ros(msg))
    }
}
