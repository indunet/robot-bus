#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/stamped.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/transform.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/transform_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::TransformStamped transform_stamped_to_bus(const ::geometry_msgs::msg::TransformStamped &msg) {
  ::geometry_msgs::msg::v1::TransformStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_child_frame_id(msg.child_frame_id.c_str());
  *bus.mutable_transform() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_to_bus(msg.transform);
  return bus;
}

inline ::geometry_msgs::msg::TransformStamped transform_stamped_to_ros(const ::geometry_msgs::msg::v1::TransformStamped &bus) {
  ::geometry_msgs::msg::TransformStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.child_frame_id = bus.child_frame_id();
  out.transform = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_to_ros(bus.transform());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsTransformStampedMapper
    : public TypedTopicMapper<GeometryMsgsTransformStampedMapper, ::geometry_msgs::msg::TransformStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::TransformStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::transform_stamped_to_bus(msg);
    return encode_pb(bus);
  }

  ::geometry_msgs::msg::TransformStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::TransformStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::transform_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsTransformStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
