#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/action_msgs/msg/v1/goal_info.pb.h>
#include <robot_bus/ros2_bridge/mappers/unique_identifier_msgs/uuid.hpp>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/time.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <action_msgs/msg/goal_info.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace action_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::action_msgs::msg::v1::GoalInfo goal_info_to_bus(const ::action_msgs::msg::GoalInfo &msg) {
  ::action_msgs::msg::v1::GoalInfo bus;
  *bus.mutable_goal_id() = ::robot_bus::ros2_bridge_mappers::unique_identifier_msgs::uuid_to_bus(msg.goal_id);
  *bus.mutable_stamp() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.stamp);
  return bus;
}

inline ::action_msgs::msg::GoalInfo goal_info_to_ros(const ::action_msgs::msg::v1::GoalInfo &bus) {
  ::action_msgs::msg::GoalInfo out;
  out.goal_id = ::robot_bus::ros2_bridge_mappers::unique_identifier_msgs::uuid_to_ros(bus.goal_id());
  out.stamp = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.stamp());
  return out;
}
#endif

}  // namespace action_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class ActionMsgsGoalInfoMapper
    : public TypedTopicMapper<ActionMsgsGoalInfoMapper, ::action_msgs::msg::GoalInfo> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::action_msgs::msg::GoalInfo &msg) const {
    auto bus = ros2_bridge_mappers::action_msgs::goal_info_to_bus(msg);
    return encode_pb(bus);
  }

  ::action_msgs::msg::GoalInfo bus_to_ros(BytesView payload) const {
    ::action_msgs::msg::v1::GoalInfo bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::action_msgs::goal_info_to_ros(bus);
  }
};
#else
struct ActionMsgsGoalInfoMapper : TopicMapper {};
#endif

}  // namespace robot_bus
