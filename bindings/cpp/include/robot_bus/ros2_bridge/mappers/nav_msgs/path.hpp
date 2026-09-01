#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/path.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose_stamped.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/path.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::Path path_to_bus(const ::nav_msgs::msg::Path &msg) {
  ::nav_msgs::msg::v1::Path bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.poses) {
    *bus.add_poses() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_bus(x);
  }
  return bus;
}

inline ::nav_msgs::msg::Path path_to_ros(const ::nav_msgs::msg::v1::Path &bus) {
  ::nav_msgs::msg::Path out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.poses.clear();
  for (const auto &x : bus.poses()) {
    out.poses.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_ros(x));
  }
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsPathMapper
    : public TypedTopicMapper<NavMsgsPathMapper, ::nav_msgs::msg::Path> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::Path &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::path_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav_msgs::msg::Path bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::Path bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::path_to_ros(bus);
  }
};
#else
struct NavMsgsPathMapper : TopicMapper {};
#endif

}  // namespace robot_bus
