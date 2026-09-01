#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/velocity.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/velocity_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::VelocityStamped velocity_stamped_to_bus(const ::geometry_msgs::msg::VelocityStamped &msg) {
  ::geometry_msgs::msg::v1::VelocityStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_body_frame_id(msg.body_frame_id.c_str());
  bus.set_reference_frame_id(msg.reference_frame_id.c_str());
  *bus.mutable_velocity() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(msg.velocity);
  return bus;
}

inline ::geometry_msgs::msg::VelocityStamped velocity_stamped_to_ros(const ::geometry_msgs::msg::v1::VelocityStamped &bus) {
  ::geometry_msgs::msg::VelocityStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.body_frame_id = bus.body_frame_id();
  out.reference_frame_id = bus.reference_frame_id();
  out.velocity = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(bus.velocity());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsVelocityStampedMapper
    : public TypedTopicMapper<GeometryMsgsVelocityStampedMapper, ::geometry_msgs::msg::VelocityStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::VelocityStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::velocity_stamped_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::VelocityStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::VelocityStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::velocity_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsVelocityStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
