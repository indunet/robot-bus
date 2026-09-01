#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/joint_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/joint_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::JointState joint_state_to_bus(const ::sensor_msgs::msg::JointState &msg) {
  ::sensor_msgs::msg::v1::JointState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.name) {
    bus.add_name(x.c_str());
  }
  for (auto x : msg.position) {
    bus.add_position(x);
  }
  for (auto x : msg.velocity) {
    bus.add_velocity(x);
  }
  for (auto x : msg.effort) {
    bus.add_effort(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::JointState joint_state_to_ros(const ::sensor_msgs::msg::v1::JointState &bus) {
  ::sensor_msgs::msg::JointState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.name.clear();
  for (const auto &x : bus.name()) {
    out.name.push_back(x);
  }
  out.position.assign(bus.position().begin(), bus.position().end());
  out.velocity.assign(bus.velocity().begin(), bus.velocity().end());
  out.effort.assign(bus.effort().begin(), bus.effort().end());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsJointStateMapper
    : public TypedTopicMapper<SensorMsgsJointStateMapper, ::sensor_msgs::msg::JointState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::JointState &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::joint_state_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::JointState bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::JointState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::joint_state_to_ros(bus);
  }
};
#else
struct SensorMsgsJointStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
