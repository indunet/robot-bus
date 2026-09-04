#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/action_msgs/msg/v1/goal_status.pb.h>
#include <robot_bus/ros2_bridge/mappers/action_msgs/goal_info.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <action_msgs/msg/goal_status.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace action_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::action_msgs::msg::v1::GoalStatus goal_status_to_bus(const ::action_msgs::msg::GoalStatus &msg) {
  ::action_msgs::msg::v1::GoalStatus bus;
  *bus.mutable_goal_info() = ::robot_bus::ros2_bridge_mappers::action_msgs::goal_info_to_bus(msg.goal_info);
  bus.set_status(static_cast<int32_t>(msg.status));
  return bus;
}

inline ::action_msgs::msg::GoalStatus goal_status_to_ros(const ::action_msgs::msg::v1::GoalStatus &bus) {
  ::action_msgs::msg::GoalStatus out;
  out.goal_info = ::robot_bus::ros2_bridge_mappers::action_msgs::goal_info_to_ros(bus.goal_info());
  out.status = static_cast<int8_t>(bus.status());
  return out;
}
#endif

}  // namespace action_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class ActionMsgsGoalStatusMapper
    : public TypedTopicMapper<ActionMsgsGoalStatusMapper, ::action_msgs::msg::GoalStatus> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::action_msgs::msg::GoalStatus &msg) const {
    auto bus = ros2_bridge_mappers::action_msgs::goal_status_to_bus(msg);
    return encode_pb(bus);
  }

  ::action_msgs::msg::GoalStatus bus_to_ros(BytesView payload) const {
    ::action_msgs::msg::v1::GoalStatus bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::action_msgs::goal_status_to_ros(bus);
  }
};
#else
struct ActionMsgsGoalStatusMapper : TopicMapper {};
#endif

}  // namespace robot_bus
