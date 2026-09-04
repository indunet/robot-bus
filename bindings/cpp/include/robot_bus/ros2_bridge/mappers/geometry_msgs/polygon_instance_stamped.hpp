#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/polygon.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/polygon_instance.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/polygon_instance_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::PolygonInstanceStamped polygon_instance_stamped_to_bus(const ::geometry_msgs::msg::PolygonInstanceStamped &msg) {
  ::geometry_msgs::msg::v1::PolygonInstanceStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_polygon() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::polygon_instance_to_bus(msg.polygon);
  return bus;
}

inline ::geometry_msgs::msg::PolygonInstanceStamped polygon_instance_stamped_to_ros(const ::geometry_msgs::msg::v1::PolygonInstanceStamped &bus) {
  ::geometry_msgs::msg::PolygonInstanceStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.polygon = ::robot_bus::ros2_bridge_mappers::geometry_msgs::polygon_instance_to_ros(bus.polygon());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPolygonInstanceStampedMapper
    : public TypedTopicMapper<GeometryMsgsPolygonInstanceStampedMapper, ::geometry_msgs::msg::PolygonInstanceStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::PolygonInstanceStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::polygon_instance_stamped_to_bus(msg);
    return encode_pb(bus);
  }

  ::geometry_msgs::msg::PolygonInstanceStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::PolygonInstanceStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::polygon_instance_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsPolygonInstanceStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
