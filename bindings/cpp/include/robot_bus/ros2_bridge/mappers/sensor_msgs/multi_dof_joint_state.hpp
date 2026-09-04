#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/multi_dof_joint_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/transform.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/wrench.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/multi_dof_joint_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::MultiDOFJointState multi_dof_joint_state_to_bus(const ::sensor_msgs::msg::MultiDOFJointState &msg) {
  ::sensor_msgs::msg::v1::MultiDOFJointState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.joint_names) {
    bus.add_joint_names(x.c_str());
  }
  for (const auto &x : msg.transforms) {
    *bus.add_transforms() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_to_bus(x);
  }
  for (const auto &x : msg.twist) {
    *bus.add_twist() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(x);
  }
  for (const auto &x : msg.wrench) {
    *bus.add_wrench() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_to_bus(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::MultiDOFJointState multi_dof_joint_state_to_ros(const ::sensor_msgs::msg::v1::MultiDOFJointState &bus) {
  ::sensor_msgs::msg::MultiDOFJointState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.joint_names.clear();
  for (const auto &x : bus.joint_names()) {
    out.joint_names.push_back(x);
  }
  out.transforms.clear();
  for (const auto &x : bus.transforms()) {
    out.transforms.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_to_ros(x));
  }
  out.twist.clear();
  for (const auto &x : bus.twist()) {
    out.twist.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(x));
  }
  out.wrench.clear();
  for (const auto &x : bus.wrench()) {
    out.wrench.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_to_ros(x));
  }
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsMultiDofJointStateMapper
    : public TypedTopicMapper<SensorMsgsMultiDofJointStateMapper, ::sensor_msgs::msg::MultiDOFJointState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::MultiDOFJointState &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::multi_dof_joint_state_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::MultiDOFJointState bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::MultiDOFJointState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::multi_dof_joint_state_to_ros(bus);
  }
};
#else
struct SensorMsgsMultiDofJointStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
