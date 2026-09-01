#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/point.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/point.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Point point_to_bus(const ::geometry_msgs::msg::Point &msg) {
  ::geometry_msgs::msg::v1::Point bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  bus.set_z(msg.z);
  return bus;
}

inline ::geometry_msgs::msg::Point point_to_ros(const ::geometry_msgs::msg::v1::Point &bus) {
  ::geometry_msgs::msg::Point out;
  out.x = bus.x();
  out.y = bus.y();
  out.z = bus.z();
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPointMapper
    : public TypedTopicMapper<GeometryMsgsPointMapper, ::geometry_msgs::msg::Point> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Point &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::point_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Point bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Point bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::point_to_ros(bus);
  }
};
#else
struct GeometryMsgsPointMapper : TopicMapper {};
#endif

}  // namespace robot_bus
