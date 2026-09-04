#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/nav_msgs/msg/v1/odometry.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose_with_covariance.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist_with_covariance.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <nav_msgs/msg/odometry.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace nav_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::nav_msgs::msg::v1::Odometry odometry_to_bus(const ::nav_msgs::msg::Odometry &msg) {
  ::nav_msgs::msg::v1::Odometry bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_child_frame_id(msg.child_frame_id.c_str());
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_with_covariance_to_bus(msg.pose);
  *bus.mutable_twist() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_with_covariance_to_bus(msg.twist);
  return bus;
}

inline ::nav_msgs::msg::Odometry odometry_to_ros(const ::nav_msgs::msg::v1::Odometry &bus) {
  ::nav_msgs::msg::Odometry out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.child_frame_id = bus.child_frame_id();
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_with_covariance_to_ros(bus.pose());
  out.twist = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_with_covariance_to_ros(bus.twist());
  return out;
}
#endif

}  // namespace nav_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class NavMsgsOdometryMapper
    : public TypedTopicMapper<NavMsgsOdometryMapper, ::nav_msgs::msg::Odometry> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::nav_msgs::msg::Odometry &msg) const {
    auto bus = ros2_bridge_mappers::nav_msgs::odometry_to_bus(msg);
    return encode_pb(bus);
  }

  ::nav_msgs::msg::Odometry bus_to_ros(BytesView payload) const {
    ::nav_msgs::msg::v1::Odometry bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::nav_msgs::odometry_to_ros(bus);
  }
};
#else
struct NavMsgsOdometryMapper : TopicMapper {};
#endif

}  // namespace robot_bus
