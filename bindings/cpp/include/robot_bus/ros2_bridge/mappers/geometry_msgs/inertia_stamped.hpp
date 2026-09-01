#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/inertia.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/inertia.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/inertia_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::InertiaStamped inertia_stamped_to_bus(const ::geometry_msgs::msg::InertiaStamped &msg) {
  ::geometry_msgs::msg::v1::InertiaStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_inertia() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::inertia_to_bus(msg.inertia);
  return bus;
}

inline ::geometry_msgs::msg::InertiaStamped inertia_stamped_to_ros(const ::geometry_msgs::msg::v1::InertiaStamped &bus) {
  ::geometry_msgs::msg::InertiaStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.inertia = ::robot_bus::ros2_bridge_mappers::geometry_msgs::inertia_to_ros(bus.inertia());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsInertiaStampedMapper
    : public TypedTopicMapper<GeometryMsgsInertiaStampedMapper, ::geometry_msgs::msg::InertiaStamped> {
 public:
  const char *type_name() const override { return "geometry_msgs/msg/InertiaStamped"; }

  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::InertiaStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::inertia_stamped_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::InertiaStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::InertiaStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::inertia_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsInertiaStampedMapper : TopicMapper {
  const char *type_name() const override { return "geometry_msgs/msg/InertiaStamped"; }
};
#endif

}  // namespace robot_bus
