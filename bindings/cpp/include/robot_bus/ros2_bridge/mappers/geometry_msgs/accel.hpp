#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/accel_wrench.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/accel.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Accel accel_to_bus(const ::geometry_msgs::msg::Accel &msg) {
  ::geometry_msgs::msg::v1::Accel bus;
  *bus.mutable_linear() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.linear);
  *bus.mutable_angular() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.angular);
  return bus;
}

inline ::geometry_msgs::msg::Accel accel_to_ros(const ::geometry_msgs::msg::v1::Accel &bus) {
  ::geometry_msgs::msg::Accel out;
  out.linear = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.linear());
  out.angular = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.angular());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsAccelMapper
    : public TypedTopicMapper<GeometryMsgsAccelMapper, ::geometry_msgs::msg::Accel> {
 public:
  const char *type_name() const override { return "geometry_msgs/msg/Accel"; }

  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Accel &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::accel_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Accel bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Accel bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::accel_to_ros(bus);
  }
};
#else
struct GeometryMsgsAccelMapper : TopicMapper {
  const char *type_name() const override { return "geometry_msgs/msg/Accel"; }
};
#endif

}  // namespace robot_bus
