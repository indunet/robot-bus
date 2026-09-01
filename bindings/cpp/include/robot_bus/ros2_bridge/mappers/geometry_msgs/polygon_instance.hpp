#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/polygon.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/polygon.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/polygon_instance.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::PolygonInstance polygon_instance_to_bus(const ::geometry_msgs::msg::PolygonInstance &msg) {
  ::geometry_msgs::msg::v1::PolygonInstance bus;
  *bus.mutable_polygon() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::polygon_to_bus(msg.polygon);
  bus.set_id(msg.id);
  return bus;
}

inline ::geometry_msgs::msg::PolygonInstance polygon_instance_to_ros(const ::geometry_msgs::msg::v1::PolygonInstance &bus) {
  ::geometry_msgs::msg::PolygonInstance out;
  out.polygon = ::robot_bus::ros2_bridge_mappers::geometry_msgs::polygon_to_ros(bus.polygon());
  out.id = bus.id();
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPolygonInstanceMapper
    : public TypedTopicMapper<GeometryMsgsPolygonInstanceMapper, ::geometry_msgs::msg::PolygonInstance> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::PolygonInstance &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::polygon_instance_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::PolygonInstance bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::PolygonInstance bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::polygon_instance_to_ros(bus);
  }
};
#else
struct GeometryMsgsPolygonInstanceMapper : TopicMapper {};
#endif

}  // namespace robot_bus
