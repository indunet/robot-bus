#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/goals.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose_stamped.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/goals.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::Goals goals_to_bus(const ::nav_msgs::msg::Goals &msg) {
  ::nav_msgs::msg::v1::Goals bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.goals) {
    *bus.add_goals() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_bus(x);
  }
  return bus;
}

inline ::nav_msgs::msg::Goals goals_to_ros(const ::nav_msgs::msg::v1::Goals &bus) {
  ::nav_msgs::msg::Goals out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.goals.clear();
  for (const auto &x : bus.goals()) {
    out.goals.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_ros(x));
  }
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsGoalsMapper
    : public TypedTopicMapper<NavMsgsGoalsMapper, ::nav_msgs::msg::Goals> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::Goals &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::goals_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav_msgs::msg::Goals bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::Goals bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::goals_to_ros(bus);
  }
};
#else
struct NavMsgsGoalsMapper : TopicMapper {};
#endif

}  // namespace robot_bus
