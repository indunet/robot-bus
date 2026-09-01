#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/stamped.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/wrench.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/wrench_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::WrenchStamped wrench_stamped_to_bus(const ::geometry_msgs::msg::WrenchStamped &msg) {
  ::geometry_msgs::msg::v1::WrenchStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_wrench() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_to_bus(msg.wrench);
  return bus;
}

inline ::geometry_msgs::msg::WrenchStamped wrench_stamped_to_ros(const ::geometry_msgs::msg::v1::WrenchStamped &bus) {
  ::geometry_msgs::msg::WrenchStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.wrench = ::robot_bus::ros2_bridge_mappers::geometry_msgs::wrench_to_ros(bus.wrench());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsWrenchStampedMapper
    : public TypedTopicMapper<GeometryMsgsWrenchStampedMapper, ::geometry_msgs::msg::WrenchStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::WrenchStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::wrench_stamped_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::WrenchStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::WrenchStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::wrench_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsWrenchStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
