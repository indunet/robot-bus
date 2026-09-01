#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/action_msgs/msg/v1/goal_status.pb.h>
#include <robot_bus/ros2_bridge/mappers/action_msgs/goal_status.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <action_msgs/msg/goal_status_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace action_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::action_msgs::msg::v1::GoalStatusArray goal_status_array_to_bus(const ::action_msgs::msg::GoalStatusArray &msg) {
  ::action_msgs::msg::v1::GoalStatusArray bus;
  for (const auto &x : msg.status_list) {
    *bus.add_status_list() = ::robot_bus::ros2_bridge_mappers::action_msgs::goal_status_to_bus(x);
  }
  return bus;
}

inline ::action_msgs::msg::GoalStatusArray goal_status_array_to_ros(const ::action_msgs::msg::v1::GoalStatusArray &bus) {
  ::action_msgs::msg::GoalStatusArray out;
  out.status_list.clear();
  for (const auto &x : bus.status_list()) {
    out.status_list.push_back(::robot_bus::ros2_bridge_mappers::action_msgs::goal_status_to_ros(x));
  }
  return out;
}
#endif

}  // namespace action_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class ActionMsgsGoalStatusArrayMapper
    : public TypedTopicMapper<ActionMsgsGoalStatusArrayMapper, ::action_msgs::msg::GoalStatusArray> {
 public:
  const char *type_name() const override { return "action_msgs/msg/GoalStatusArray"; }

  std::vector<uint8_t> ros_to_bus(const ::action_msgs::msg::GoalStatusArray &msg) const {
    auto bus = ros2_bridge_mappers::action_msgs::goal_status_array_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::action_msgs::msg::GoalStatusArray bus_to_ros(BytesView payload) const {
    ::action_msgs::msg::v1::GoalStatusArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::action_msgs::goal_status_array_to_ros(bus);
  }
};
#else
struct ActionMsgsGoalStatusArrayMapper : TopicMapper {
  const char *type_name() const override { return "action_msgs/msg/GoalStatusArray"; }
};
#endif

}  // namespace robot_bus
