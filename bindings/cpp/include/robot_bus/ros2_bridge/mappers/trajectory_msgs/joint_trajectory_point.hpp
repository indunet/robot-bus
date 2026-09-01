#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/trajectory_msgs/msg/v1/joint_trajectory.pb.h>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/duration.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <trajectory_msgs/msg/joint_trajectory_point.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace trajectory_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::trajectory_msgs::msg::v1::JointTrajectoryPoint joint_trajectory_point_to_bus(const ::trajectory_msgs::msg::JointTrajectoryPoint &msg) {
  ::trajectory_msgs::msg::v1::JointTrajectoryPoint bus;
  for (auto x : msg.positions) {
    bus.add_positions(x);
  }
  for (auto x : msg.velocities) {
    bus.add_velocities(x);
  }
  for (auto x : msg.accelerations) {
    bus.add_accelerations(x);
  }
  for (auto x : msg.effort) {
    bus.add_effort(x);
  }
  *bus.mutable_time_from_start() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_bus(msg.time_from_start);
  return bus;
}

inline ::trajectory_msgs::msg::JointTrajectoryPoint joint_trajectory_point_to_ros(const ::trajectory_msgs::msg::v1::JointTrajectoryPoint &bus) {
  ::trajectory_msgs::msg::JointTrajectoryPoint out;
  out.positions.assign(bus.positions().begin(), bus.positions().end());
  out.velocities.assign(bus.velocities().begin(), bus.velocities().end());
  out.accelerations.assign(bus.accelerations().begin(), bus.accelerations().end());
  out.effort.assign(bus.effort().begin(), bus.effort().end());
  out.time_from_start = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::duration_to_ros(bus.time_from_start());
  return out;
}
#endif

}  // namespace trajectory_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class TrajectoryMsgsJointTrajectoryPointMapper
    : public TypedTopicMapper<TrajectoryMsgsJointTrajectoryPointMapper, ::trajectory_msgs::msg::JointTrajectoryPoint> {
 public:
  const char *type_name() const override { return "trajectory_msgs/msg/JointTrajectoryPoint"; }

  std::vector<uint8_t> ros_to_bus(const ::trajectory_msgs::msg::JointTrajectoryPoint &msg) const {
    auto bus = ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::trajectory_msgs::msg::JointTrajectoryPoint bus_to_ros(BytesView payload) const {
    ::trajectory_msgs::msg::v1::JointTrajectoryPoint bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::trajectory_msgs::joint_trajectory_point_to_ros(bus);
  }
};
#else
struct TrajectoryMsgsJointTrajectoryPointMapper : TopicMapper {
  const char *type_name() const override { return "trajectory_msgs/msg/JointTrajectoryPoint"; }
};
#endif

}  // namespace robot_bus
