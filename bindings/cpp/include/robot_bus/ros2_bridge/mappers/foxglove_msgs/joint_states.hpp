#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/joint_states.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/joint_state.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/joint_states.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::JointStates joint_states_to_bus(const ::foxglove_msgs::msg::JointStates &msg) {
  ::foxglove_msgs::msg::v1::JointStates bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  for (const auto &x : msg.joints) {
    *bus.add_joints() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::joint_state_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::JointStates joint_states_to_ros(const ::foxglove_msgs::msg::v1::JointStates &bus) {
  ::foxglove_msgs::msg::JointStates out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.joints.clear();
  for (const auto &x : bus.joints()) {
    out.joints.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::joint_state_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsJointStatesMapper
    : public TypedTopicMapper<FoxgloveMsgsJointStatesMapper, ::foxglove_msgs::msg::JointStates> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::JointStates &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::joint_states_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::JointStates bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::JointStates bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::joint_states_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsJointStatesMapper : TopicMapper {};
#endif

}  // namespace robot_bus
