#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/stamped.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/pose_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::PoseStamped pose_stamped_to_bus(const ::geometry_msgs::msg::PoseStamped &msg) {
  ::geometry_msgs::msg::v1::PoseStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  return bus;
}

inline ::geometry_msgs::msg::PoseStamped pose_stamped_to_ros(const ::geometry_msgs::msg::v1::PoseStamped &bus) {
  ::geometry_msgs::msg::PoseStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPoseStampedMapper
    : public TypedTopicMapper<GeometryMsgsPoseStampedMapper, ::geometry_msgs::msg::PoseStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::PoseStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::pose_stamped_to_bus(msg);
    return encode_pb(bus);
  }

  ::geometry_msgs::msg::PoseStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::PoseStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::pose_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsPoseStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
