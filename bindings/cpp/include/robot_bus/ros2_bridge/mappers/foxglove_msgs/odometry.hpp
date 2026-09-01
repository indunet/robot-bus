#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/odometry.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/key_value_pair.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/odometry.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Odometry odometry_to_bus(const ::foxglove_msgs::msg::Odometry &msg) {
  ::foxglove_msgs::msg::v1::Odometry bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  bus.set_body_frame_id(msg.body_frame_id.c_str());
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  *bus.mutable_linear_velocity() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.linear_velocity);
  *bus.mutable_angular_velocity() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.angular_velocity);
  for (auto x : msg.pose_covariance) {
    bus.add_pose_covariance(x);
  }
  for (auto x : msg.velocity_covariance) {
    bus.add_velocity_covariance(x);
  }
  for (const auto &x : msg.metadata) {
    *bus.add_metadata() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::Odometry odometry_to_ros(const ::foxglove_msgs::msg::v1::Odometry &bus) {
  ::foxglove_msgs::msg::Odometry out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.body_frame_id = bus.body_frame_id();
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.linear_velocity = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.linear_velocity());
  out.angular_velocity = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.angular_velocity());
  out.pose_covariance.assign(bus.pose_covariance().begin(), bus.pose_covariance().end());
  out.velocity_covariance.assign(bus.velocity_covariance().begin(), bus.velocity_covariance().end());
  out.metadata.clear();
  for (const auto &x : bus.metadata()) {
    out.metadata.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsOdometryMapper
    : public TypedTopicMapper<FoxgloveMsgsOdometryMapper, ::foxglove_msgs::msg::Odometry> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Odometry &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::odometry_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Odometry bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Odometry bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::odometry_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsOdometryMapper : TopicMapper {};
#endif

}  // namespace robot_bus
