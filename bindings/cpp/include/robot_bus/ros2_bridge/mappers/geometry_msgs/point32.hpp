#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/point32.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/point32.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Point32 point32_to_bus(const ::geometry_msgs::msg::Point32 &msg) {
  ::geometry_msgs::msg::v1::Point32 bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  bus.set_z(msg.z);
  return bus;
}

inline ::geometry_msgs::msg::Point32 point32_to_ros(const ::geometry_msgs::msg::v1::Point32 &bus) {
  ::geometry_msgs::msg::Point32 out;
  out.x = bus.x();
  out.y = bus.y();
  out.z = bus.z();
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPoint32Mapper
    : public TypedTopicMapper<GeometryMsgsPoint32Mapper, ::geometry_msgs::msg::Point32> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Point32 &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::point32_to_bus(msg);
    return encode_pb(bus);
  }

  ::geometry_msgs::msg::Point32 bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Point32 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::point32_to_ros(bus);
  }
};
#else
struct GeometryMsgsPoint32Mapper : TopicMapper {};
#endif

}  // namespace robot_bus
