#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav2_msgs/msg/v1/behavior_tree.pb.h>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/time.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
#include <nav2_msgs/msg/behavior_tree_status_change.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
inline ::nav2_msgs::msg::v1::BehaviorTreeStatusChange behavior_tree_status_change_to_bus(const ::nav2_msgs::msg::BehaviorTreeStatusChange &msg) {
  ::nav2_msgs::msg::v1::BehaviorTreeStatusChange bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.timestamp);
  bus.set_node_name(msg.node_name.c_str());
  bus.set_previous_status(msg.previous_status.c_str());
  bus.set_current_status(msg.current_status.c_str());
  return bus;
}

inline ::nav2_msgs::msg::BehaviorTreeStatusChange behavior_tree_status_change_to_ros(const ::nav2_msgs::msg::v1::BehaviorTreeStatusChange &bus) {
  ::nav2_msgs::msg::BehaviorTreeStatusChange out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.timestamp());
  out.node_name = bus.node_name();
  out.previous_status = bus.previous_status();
  out.current_status = bus.current_status();
  return out;
}
#endif

}  // namespace nav2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_NAV2_MSGS)
class Nav2MsgsBehaviorTreeStatusChangeMapper
    : public TypedTopicMapper<Nav2MsgsBehaviorTreeStatusChangeMapper, ::nav2_msgs::msg::BehaviorTreeStatusChange> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav2_msgs::msg::BehaviorTreeStatusChange &msg) const {
    auto bus = ros2_bridge_mappers::nav2_msgs::behavior_tree_status_change_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::nav2_msgs::msg::BehaviorTreeStatusChange bus_to_ros(BytesView payload) const {
    ::nav2_msgs::msg::v1::BehaviorTreeStatusChange bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav2_msgs::behavior_tree_status_change_to_ros(bus);
  }
};
#else
struct Nav2MsgsBehaviorTreeStatusChangeMapper : TopicMapper {};
#endif

}  // namespace robot_bus
