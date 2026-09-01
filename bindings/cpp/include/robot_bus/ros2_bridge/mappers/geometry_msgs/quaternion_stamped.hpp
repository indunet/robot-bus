#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/stamped.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/quaternion.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/quaternion_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::QuaternionStamped quaternion_stamped_to_bus(const ::geometry_msgs::msg::QuaternionStamped &msg) {
  ::geometry_msgs::msg::v1::QuaternionStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_quaternion() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_bus(msg.quaternion);
  return bus;
}

inline ::geometry_msgs::msg::QuaternionStamped quaternion_stamped_to_ros(const ::geometry_msgs::msg::v1::QuaternionStamped &bus) {
  ::geometry_msgs::msg::QuaternionStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.quaternion = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_ros(bus.quaternion());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsQuaternionStampedMapper
    : public TypedTopicMapper<GeometryMsgsQuaternionStampedMapper, ::geometry_msgs::msg::QuaternionStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::QuaternionStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::quaternion_stamped_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::QuaternionStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::QuaternionStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::quaternion_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsQuaternionStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
