#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/trajectory.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/nav_msgs/trajectory_point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/trajectory.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::Trajectory trajectory_to_bus(const ::nav_msgs::msg::Trajectory &msg) {
  ::nav_msgs::msg::v1::Trajectory bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::nav_msgs::trajectory_point_to_bus(x);
  }
  return bus;
}

inline ::nav_msgs::msg::Trajectory trajectory_to_ros(const ::nav_msgs::msg::v1::Trajectory &bus) {
  ::nav_msgs::msg::Trajectory out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::nav_msgs::trajectory_point_to_ros(x));
  }
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsTrajectoryMapper
    : public TypedTopicMapper<NavMsgsTrajectoryMapper, ::nav_msgs::msg::Trajectory> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::Trajectory &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::trajectory_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav_msgs::msg::Trajectory bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::Trajectory bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::trajectory_to_ros(bus);
  }
};
#else
struct NavMsgsTrajectoryMapper : TopicMapper {};
#endif

}  // namespace robot_bus
