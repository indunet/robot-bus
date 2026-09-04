#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/control_msgs/msg/v1/joint_trajectory_controller_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/trajectory_msgs/joint_trajectory_point.hpp>
#include <robot_bus/ros2_bridge/mappers/trajectory_msgs/multi_dof_joint_trajectory_point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
#include <control_msgs/msg/joint_trajectory_controller_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace control_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
inline ::control_msgs::msg::v1::JointTrajectoryControllerState joint_trajectory_controller_state_to_bus(const ::control_msgs::msg::JointTrajectoryControllerState &msg) {
  ::control_msgs::msg::v1::JointTrajectoryControllerState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.joint_names) {
    bus.add_joint_names(x.c_str());
  }
  *bus.mutable_reference() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_bus(msg.reference);
  *bus.mutable_feedback() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_bus(msg.feedback);
  *bus.mutable_error() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_bus(msg.error);
  *bus.mutable_output() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_bus(msg.output);
  for (const auto &x : msg.multi_dof_joint_names) {
    bus.add_multi_dof_joint_names(x.c_str());
  }
  *bus.mutable_multi_dof_reference() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_reference);
  *bus.mutable_multi_dof_feedback() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_feedback);
  *bus.mutable_multi_dof_error() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_error);
  *bus.mutable_multi_dof_output() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_bus(msg.multi_dof_output);
  return bus;
}

inline ::control_msgs::msg::JointTrajectoryControllerState joint_trajectory_controller_state_to_ros(const ::control_msgs::msg::v1::JointTrajectoryControllerState &bus) {
  ::control_msgs::msg::JointTrajectoryControllerState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.joint_names.clear();
  for (const auto &x : bus.joint_names()) {
    out.joint_names.push_back(x);
  }
  out.reference = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_ros(bus.reference());
  out.feedback = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_ros(bus.feedback());
  out.error = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_ros(bus.error());
  out.output = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_ros(bus.output());
  out.multi_dof_joint_names.clear();
  for (const auto &x : bus.multi_dof_joint_names()) {
    out.multi_dof_joint_names.push_back(x);
  }
  out.multi_dof_reference = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_reference());
  out.multi_dof_feedback = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_feedback());
  out.multi_dof_error = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_error());
  out.multi_dof_output = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_ros(bus.multi_dof_output());
  return out;
}
#endif

}  // namespace control_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_CONTROL_MSGS)
class ControlMsgsJointTrajectoryControllerStateMapper
    : public TypedTopicMapper<ControlMsgsJointTrajectoryControllerStateMapper, ::control_msgs::msg::JointTrajectoryControllerState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::control_msgs::msg::JointTrajectoryControllerState &msg) const {
    auto bus = ros2_bridge_mappers::control_msgs::joint_trajectory_controller_state_to_bus(msg);
    return encode_pb(bus);
  }

  ::control_msgs::msg::JointTrajectoryControllerState bus_to_ros(BytesView payload) const {
    ::control_msgs::msg::v1::JointTrajectoryControllerState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::control_msgs::joint_trajectory_controller_state_to_ros(bus);
  }
};
#else
struct ControlMsgsJointTrajectoryControllerStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
