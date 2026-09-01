#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/joint_state.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/joint_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::JointState joint_state_to_bus(const ::foxglove_msgs::msg::JointState &msg) {
  ::foxglove_msgs::msg::v1::JointState bus;
  bus.set_name(msg.name.c_str());
  bus.set_position(msg.position);
  bus.set_velocity(msg.velocity);
  bus.set_acceleration(msg.acceleration);
  bus.set_effort(msg.effort);
  return bus;
}

inline ::foxglove_msgs::msg::JointState joint_state_to_ros(const ::foxglove_msgs::msg::v1::JointState &bus) {
  ::foxglove_msgs::msg::JointState out;
  out.name = bus.name();
  out.position = bus.position();
  out.velocity = bus.velocity();
  out.acceleration = bus.acceleration();
  out.effort = bus.effort();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsJointStateMapper
    : public TypedTopicMapper<FoxgloveMsgsJointStateMapper, ::foxglove_msgs::msg::JointState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::JointState &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::joint_state_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::JointState bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::JointState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::joint_state_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsJointStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
