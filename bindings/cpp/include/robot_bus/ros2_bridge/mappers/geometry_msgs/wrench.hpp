#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/accel_wrench.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/wrench.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Wrench wrench_to_bus(const ::geometry_msgs::msg::Wrench &msg) {
  ::geometry_msgs::msg::v1::Wrench bus;
  *bus.mutable_force() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.force);
  *bus.mutable_torque() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.torque);
  return bus;
}

inline ::geometry_msgs::msg::Wrench wrench_to_ros(const ::geometry_msgs::msg::v1::Wrench &bus) {
  ::geometry_msgs::msg::Wrench out;
  out.force = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.force());
  out.torque = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.torque());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsWrenchMapper
    : public TypedTopicMapper<GeometryMsgsWrenchMapper, ::geometry_msgs::msg::Wrench> {
 public:
  const char *type_name() const override { return "geometry_msgs/msg/Wrench"; }

  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Wrench &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::wrench_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Wrench bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Wrench bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::wrench_to_ros(bus);
  }
};
#else
struct GeometryMsgsWrenchMapper : TopicMapper {
  const char *type_name() const override { return "geometry_msgs/msg/Wrench"; }
};
#endif

}  // namespace robot_bus
