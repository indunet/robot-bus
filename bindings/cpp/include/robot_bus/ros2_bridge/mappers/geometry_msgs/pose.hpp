#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/pose.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/quaternion.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/pose.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Pose pose_to_bus(const ::geometry_msgs::msg::Pose &msg) {
  ::geometry_msgs::msg::v1::Pose bus;
  *bus.mutable_position() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_bus(msg.position);
  *bus.mutable_orientation() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_bus(msg.orientation);
  return bus;
}

inline ::geometry_msgs::msg::Pose pose_to_ros(const ::geometry_msgs::msg::v1::Pose &bus) {
  ::geometry_msgs::msg::Pose out;
  out.position = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point_to_ros(bus.position());
  out.orientation = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_ros(bus.orientation());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPoseMapper
    : public TypedTopicMapper<GeometryMsgsPoseMapper, ::geometry_msgs::msg::Pose> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Pose &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Pose bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Pose bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus);
  }
};
#else
struct GeometryMsgsPoseMapper : TopicMapper {};
#endif

}  // namespace robot_bus
