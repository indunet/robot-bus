#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/polygon.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/point32.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/polygon.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Polygon polygon_to_bus(const ::geometry_msgs::msg::Polygon &msg) {
  ::geometry_msgs::msg::v1::Polygon bus;
  for (const auto &x : msg.points) {
    *bus.add_points() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::point32_to_bus(x);
  }
  return bus;
}

inline ::geometry_msgs::msg::Polygon polygon_to_ros(const ::geometry_msgs::msg::v1::Polygon &bus) {
  ::geometry_msgs::msg::Polygon out;
  out.points.clear();
  for (const auto &x : bus.points()) {
    out.points.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::point32_to_ros(x));
  }
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPolygonMapper
    : public TypedTopicMapper<GeometryMsgsPolygonMapper, ::geometry_msgs::msg::Polygon> {
 public:
  const char *type_name() const override { return "geometry_msgs/msg/Polygon"; }

  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Polygon &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::polygon_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Polygon bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Polygon bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::polygon_to_ros(bus);
  }
};
#else
struct GeometryMsgsPolygonMapper : TopicMapper {
  const char *type_name() const override { return "geometry_msgs/msg/Polygon"; }
};
#endif

}  // namespace robot_bus
