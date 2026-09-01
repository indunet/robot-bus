#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/transform.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/quaternion.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/transform.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Transform transform_to_bus(const ::geometry_msgs::msg::Transform &msg) {
  ::geometry_msgs::msg::v1::Transform bus;
  *bus.mutable_translation() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.translation);
  *bus.mutable_rotation() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_bus(msg.rotation);
  return bus;
}

inline ::geometry_msgs::msg::Transform transform_to_ros(const ::geometry_msgs::msg::v1::Transform &bus) {
  ::geometry_msgs::msg::Transform out;
  out.translation = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.translation());
  out.rotation = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_ros(bus.rotation());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsTransformMapper
    : public TypedTopicMapper<GeometryMsgsTransformMapper, ::geometry_msgs::msg::Transform> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Transform &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::transform_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Transform bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Transform bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::transform_to_ros(bus);
  }
};
#else
struct GeometryMsgsTransformMapper : TopicMapper {};
#endif

}  // namespace robot_bus
