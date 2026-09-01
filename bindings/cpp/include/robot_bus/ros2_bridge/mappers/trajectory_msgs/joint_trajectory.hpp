#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/trajectory_msgs/msg/v1/joint_trajectory.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/trajectory_msgs/joint_trajectory_point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <trajectory_msgs/msg/joint_trajectory.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace trajectory_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::trajectory_msgs::msg::v1::JointTrajectory joint_trajectory_to_bus(const ::trajectory_msgs::msg::JointTrajectory &msg) {
  ::trajectory_msgs::msg::v1::JointTrajectory bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.joint_names) {
    bus.add_joint_names(x.c_str());
  }
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_bus(x);
  }
  return bus;
}

inline ::trajectory_msgs::msg::JointTrajectory joint_trajectory_to_ros(const ::trajectory_msgs::msg::v1::JointTrajectory &bus) {
  ::trajectory_msgs::msg::JointTrajectory out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.joint_names.clear();
  for (const auto &x : bus.joint_names()) {
    out.joint_names.push_back(x);
  }
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_ros(x));
  }
  return out;
}
#endif

}  // namespace trajectory_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class TrajectoryMsgsJointTrajectoryMapper
    : public TypedTopicMapper<TrajectoryMsgsJointTrajectoryMapper, ::trajectory_msgs::msg::JointTrajectory> {
 public:
  const char *type_name() const override { return "trajectory_msgs/msg/JointTrajectory"; }

  std::vector<uint8_t> ros_to_bus(const ::trajectory_msgs::msg::JointTrajectory &msg) const {
    auto bus = ros2_bridge_mappers::trajectory_msgs::joint_trajectory_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::trajectory_msgs::msg::JointTrajectory bus_to_ros(BytesView payload) const {
    ::trajectory_msgs::msg::v1::JointTrajectory bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::trajectory_msgs::joint_trajectory_to_ros(bus);
  }
};
#else
struct TrajectoryMsgsJointTrajectoryMapper : TopicMapper {
  const char *type_name() const override { return "trajectory_msgs/msg/JointTrajectory"; }
};
#endif

}  // namespace robot_bus
