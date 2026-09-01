#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/pose_array.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/pose_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::PoseArray pose_array_to_bus(const ::geometry_msgs::msg::PoseArray &msg) {
  ::geometry_msgs::msg::v1::PoseArray bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.poses) {
    *bus.add_poses() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(x);
  }
  return bus;
}

inline ::geometry_msgs::msg::PoseArray pose_array_to_ros(const ::geometry_msgs::msg::v1::PoseArray &bus) {
  ::geometry_msgs::msg::PoseArray out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.poses.clear();
  for (const auto &x : bus.poses()) {
    out.poses.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(x));
  }
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPoseArrayMapper
    : public TypedTopicMapper<GeometryMsgsPoseArrayMapper, ::geometry_msgs::msg::PoseArray> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::PoseArray &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::pose_array_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::PoseArray bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::PoseArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::pose_array_to_ros(bus);
  }
};
#else
struct GeometryMsgsPoseArrayMapper : TopicMapper {};
#endif

}  // namespace robot_bus
