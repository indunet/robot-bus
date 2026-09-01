#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/particle_cloud.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/nav2_msgs/particle.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/particle_cloud.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::ParticleCloud particle_cloud_to_bus(const ::nav2_msgs::msg::ParticleCloud &msg) {
  ::nav2_msgs::msg::v1::ParticleCloud bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.particles) {
    *bus.add_particles() = ::robot_bus::ros2_bridge_mappers::nav2_msgs::particle_to_bus(x);
  }
  return bus;
}

inline ::nav2_msgs::msg::ParticleCloud particle_cloud_to_ros(const ::nav2_msgs::msg::v1::ParticleCloud &bus) {
  ::nav2_msgs::msg::ParticleCloud out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.particles.clear();
  for (const auto &x : bus.particles()) {
    out.particles.push_back(::robot_bus::ros2_bridge_mappers::nav2_msgs::particle_to_ros(x));
  }
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsParticleCloudMapper
    : public TypedTopicMapper<Nav2MsgsParticleCloudMapper, ::nav2_msgs::msg::ParticleCloud> {
 public:
  const char *type_name() const override { return "nav2_msgs/msg/ParticleCloud"; }

  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::ParticleCloud &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::particle_cloud_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::ParticleCloud bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::ParticleCloud bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::particle_cloud_to_ros(bus);
  }
};
#else
struct Nav2MsgsParticleCloudMapper : TopicMapper {
  const char *type_name() const override { return "nav2_msgs/msg/ParticleCloud"; }
};
#endif

}  // namespace robot_bus
