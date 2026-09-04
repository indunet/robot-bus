#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/behavior_tree.pb.h>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/time.hpp>
#include <robot_bus/ros2_bridge/mappers/nav2_msgs/behavior_tree_status_change.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/behavior_tree_log.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::BehaviorTreeLog behavior_tree_log_to_bus(const ::nav2_msgs::msg::BehaviorTreeLog &msg) {
  ::nav2_msgs::msg::v1::BehaviorTreeLog bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.timestamp);
  for (const auto &x : msg.event_log) {
    *bus.add_event_log() = ::robot_bus::ros2_bridge_mappers::nav2_msgs::behavior_tree_status_change_to_bus(x);
  }
  return bus;
}

inline ::nav2_msgs::msg::BehaviorTreeLog behavior_tree_log_to_ros(const ::nav2_msgs::msg::v1::BehaviorTreeLog &bus) {
  ::nav2_msgs::msg::BehaviorTreeLog out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.timestamp());
  out.event_log.clear();
  for (const auto &x : bus.event_log()) {
    out.event_log.push_back(::robot_bus::ros2_bridge_mappers::nav2_msgs::behavior_tree_status_change_to_ros(x));
  }
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsBehaviorTreeLogMapper
    : public TypedTopicMapper<Nav2MsgsBehaviorTreeLogMapper, ::nav2_msgs::msg::BehaviorTreeLog> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::BehaviorTreeLog &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::behavior_tree_log_to_bus(msg);
    return encode_pb(bus);
  }

  ::nav2_msgs::msg::BehaviorTreeLog bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::BehaviorTreeLog bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::behavior_tree_log_to_ros(bus);
  }
};
#else
struct Nav2MsgsBehaviorTreeLogMapper : TopicMapper {};
#endif

}  // namespace robot_bus
