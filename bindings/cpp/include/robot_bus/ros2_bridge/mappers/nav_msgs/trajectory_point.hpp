#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/trajectory.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/accel.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/wrench.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/trajectory_point.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::TrajectoryPoint trajectory_point_to_bus(const ::nav_msgs::msg::TrajectoryPoint &msg) {
  ::nav_msgs::msg::v1::TrajectoryPoint bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  *bus.mutable_velocity() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(msg.velocity);
  *bus.mutable_acceleration() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::accel_to_bus(msg.acceleration);
  *bus.mutable_effort() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_to_bus(msg.effort);
  return bus;
}

inline ::nav_msgs::msg::TrajectoryPoint trajectory_point_to_ros(const ::nav_msgs::msg::v1::TrajectoryPoint &bus) {
  ::nav_msgs::msg::TrajectoryPoint out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  out.velocity = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(bus.velocity());
  out.acceleration = ::robot_bus::ros2_bridge_mappers::geometry_msgs::accel_to_ros(bus.acceleration());
  out.effort = ::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_to_ros(bus.effort());
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsTrajectoryPointMapper
    : public TypedTopicMapper<NavMsgsTrajectoryPointMapper, ::nav_msgs::msg::TrajectoryPoint> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::TrajectoryPoint &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::trajectory_point_to_bus(msg);
    return encode_pb(bus);
  }

  ::nav_msgs::msg::TrajectoryPoint bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::TrajectoryPoint bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::trajectory_point_to_ros(bus);
  }
};
#else
struct NavMsgsTrajectoryPointMapper : TopicMapper {};
#endif

}  // namespace robot_bus
