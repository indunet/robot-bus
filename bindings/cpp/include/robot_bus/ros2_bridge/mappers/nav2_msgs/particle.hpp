#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/particle_cloud.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/particle.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::Particle particle_to_bus(const ::nav2_msgs::msg::Particle &msg) {
  ::nav2_msgs::msg::v1::Particle bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  bus.set_weight(msg.weight);
  return bus;
}

inline ::nav2_msgs::msg::Particle particle_to_ros(const ::nav2_msgs::msg::v1::Particle &bus) {
  ::nav2_msgs::msg::Particle out;
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  out.weight = bus.weight();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsParticleMapper
    : public TypedTopicMapper<Nav2MsgsParticleMapper, ::nav2_msgs::msg::Particle> {
 public:
  const char *type_name() const override { return "nav2_msgs/msg/Particle"; }

  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::Particle &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::particle_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::Particle bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::Particle bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::particle_to_ros(bus);
  }
};
#else
struct Nav2MsgsParticleMapper : TopicMapper {
  const char *type_name() const override { return "nav2_msgs/msg/Particle"; }
};
#endif

}  // namespace robot_bus
