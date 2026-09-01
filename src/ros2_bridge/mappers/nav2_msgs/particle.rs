//! Typed mapper for `nav2_msgs/msg/Particle`.

use crate::ros2_bridge::mapper::TypedTopicMapper;

pub(crate) fn particle_to_bus(msg: ros_env::nav2_msgs::msg::Particle) -> crate::nav2_msgs::msg::v1::Particle {
    crate::nav2_msgs::msg::v1::Particle {
        pose: Some(crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_bus(msg.pose)),
        weight: msg.weight,
    }
}

pub(crate) fn particle_to_ros(bus: crate::nav2_msgs::msg::v1::Particle) -> ros_env::nav2_msgs::msg::Particle {
    ros_env::nav2_msgs::msg::Particle {
        pose: crate::ros2_bridge::mappers::geometry_msgs::pose::pose_to_ros(bus.pose.unwrap_or_default()),
        weight: bus.weight,
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Nav2MsgsParticleMapper;

impl TypedTopicMapper for Nav2MsgsParticleMapper {
    type Ros = ros_env::nav2_msgs::msg::Particle;
    type Bus = crate::nav2_msgs::msg::v1::Particle;

    fn ros_to_bus(&self, msg: Self::Ros) -> crate::errors::Result<Self::Bus> {
        Ok(particle_to_bus(msg))
    }

    fn bus_to_ros(&self, msg: Self::Bus) -> crate::errors::Result<Self::Ros> {
        Ok(particle_to_ros(msg))
    }
}
