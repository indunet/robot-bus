#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/trajectory_msgs/msg/v1/multi_dof_joint_trajectory.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/trajectory_msgs/multi_dof_joint_trajectory_point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <trajectory_msgs/msg/multi_dof_joint_trajectory.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace trajectory_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::trajectory_msgs::msg::v1::MultiDOFJointTrajectory multi_dof_joint_trajectory_to_bus(const ::trajectory_msgs::msg::MultiDOFJointTrajectory &msg) {
  ::trajectory_msgs::msg::v1::MultiDOFJointTrajectory bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.joint_names) {
    bus.add_joint_names(x.c_str());
  }
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_bus(x);
  }
  return bus;
}

inline ::trajectory_msgs::msg::MultiDOFJointTrajectory multi_dof_joint_trajectory_to_ros(const ::trajectory_msgs::msg::v1::MultiDOFJointTrajectory &bus) {
  ::trajectory_msgs::msg::MultiDOFJointTrajectory out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.joint_names.clear();
  for (const auto &x : bus.joint_names()) {
    out.joint_names.push_back(x);
  }
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_ros(x));
  }
  return out;
}
#endif

}  // namespace trajectory_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class TrajectoryMsgsMultiDofJointTrajectoryMapper
    : public TypedTopicMapper<TrajectoryMsgsMultiDofJointTrajectoryMapper, ::trajectory_msgs::msg::MultiDOFJointTrajectory> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::trajectory_msgs::msg::MultiDOFJointTrajectory &msg) const {
    auto bus = ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_to_bus(msg);
    return encode_pb(bus);
  }

  ::trajectory_msgs::msg::MultiDOFJointTrajectory bus_to_ros(BytesView payload) const {
    ::trajectory_msgs::msg::v1::MultiDOFJointTrajectory bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_to_ros(bus);
  }
};
#else
struct TrajectoryMsgsMultiDofJointTrajectoryMapper : TopicMapper {};
#endif

}  // namespace robot_bus
