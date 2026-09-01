#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/missed_waypoint.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose_stamped.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/missed_waypoint.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::MissedWaypoint missed_waypoint_to_bus(const ::nav2_msgs::msg::MissedWaypoint &msg) {
  ::nav2_msgs::msg::v1::MissedWaypoint bus;
  bus.set_index(msg.index);
  *bus.mutable_goal() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_bus(msg.goal);
  bus.set_error_code(msg.error_code);
  return bus;
}

inline ::nav2_msgs::msg::MissedWaypoint missed_waypoint_to_ros(const ::nav2_msgs::msg::v1::MissedWaypoint &bus) {
  ::nav2_msgs::msg::MissedWaypoint out;
  out.index = bus.index();
  out.goal = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_stamped_to_ros(bus.goal());
  out.error_code = bus.error_code();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsMissedWaypointMapper
    : public TypedTopicMapper<Nav2MsgsMissedWaypointMapper, ::nav2_msgs::msg::MissedWaypoint> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::MissedWaypoint &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::missed_waypoint_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::MissedWaypoint bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::MissedWaypoint bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::missed_waypoint_to_ros(bus);
  }
};
#else
struct Nav2MsgsMissedWaypointMapper : TopicMapper {};
#endif

}  // namespace robot_bus
