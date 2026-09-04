#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/trajectory_msgs/msg/v1/multi_dof_joint_trajectory.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/transform.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist.hpp>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/duration.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <trajectory_msgs/msg/multi_dof_joint_trajectory_point.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace trajectory_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::trajectory_msgs::msg::v1::MultiDOFJointTrajectoryPoint multi_dof_joint_trajectory_point_to_bus(const ::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint &msg) {
  ::trajectory_msgs::msg::v1::MultiDOFJointTrajectoryPoint bus;
  for (const auto &x : msg.transforms) {
    *bus.add_transforms() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_to_bus(x);
  }
  for (const auto &x : msg.velocities) {
    *bus.add_velocities() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(x);
  }
  for (const auto &x : msg.accelerations) {
    *bus.add_accelerations() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(x);
  }
  *bus.mutable_time_from_start() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_bus(msg.time_from_start);
  return bus;
}

inline ::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint multi_dof_joint_trajectory_point_to_ros(const ::trajectory_msgs::msg::v1::MultiDOFJointTrajectoryPoint &bus) {
  ::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint out;
  out.transforms.clear();
  for (const auto &x : bus.transforms()) {
    out.transforms.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_to_ros(x));
  }
  out.velocities.clear();
  for (const auto &x : bus.velocities()) {
    out.velocities.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(x));
  }
  out.accelerations.clear();
  for (const auto &x : bus.accelerations()) {
    out.accelerations.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(x));
  }
  out.time_from_start = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_ros(bus.time_from_start());
  return out;
}
#endif

}  // namespace trajectory_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class TrajectoryMsgsMultiDofJointTrajectoryPointMapper
    : public TypedTopicMapper<TrajectoryMsgsMultiDofJointTrajectoryPointMapper, ::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint &msg) const {
    auto bus = ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_bus(msg);
    return encode_pb(bus);
  }

  ::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint bus_to_ros(BytesView payload) const {
    ::trajectory_msgs::msg::v1::MultiDOFJointTrajectoryPoint bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::trajectory_msgs::multi_dof_joint_trajectory_point_to_ros(bus);
  }
};
#else
struct TrajectoryMsgsMultiDofJointTrajectoryPointMapper : TopicMapper {};
#endif

}  // namespace robot_bus
