#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/joint_jog.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/joint_jog.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::JointJog joint_jog_to_bus(const ::control_msgs::msg::JointJog &msg) {
  ::control_msgs::msg::v1::JointJog bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.joint_names) {
    bus.add_joint_names(x.c_str());
  }
  for (auto x : msg.displacements) {
    bus.add_displacements(x);
  }
  for (auto x : msg.velocities) {
    bus.add_velocities(x);
  }
  bus.set_duration(msg.duration);
  return bus;
}

inline ::control_msgs::msg::JointJog joint_jog_to_ros(const ::control_msgs::msg::v1::JointJog &bus) {
  ::control_msgs::msg::JointJog out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.joint_names.clear();
  for (const auto &x : bus.joint_names()) {
    out.joint_names.push_back(x);
  }
  out.displacements.assign(bus.displacements().begin(), bus.displacements().end());
  out.velocities.assign(bus.velocities().begin(), bus.velocities().end());
  out.duration = bus.duration();
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsJointJogMapper
    : public TypedTopicMapper<ControlMsgsJointJogMapper, ::control_msgs::msg::JointJog> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::JointJog &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::joint_jog_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::control_msgs::msg::JointJog bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::JointJog bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::joint_jog_to_ros(bus);
  }
};
#else
struct ControlMsgsJointJogMapper : TopicMapper {};
#endif

}  // namespace robot_bus
