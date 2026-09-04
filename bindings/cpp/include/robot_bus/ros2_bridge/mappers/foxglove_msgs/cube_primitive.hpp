#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/cube_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/cube_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::CubePrimitive cube_primitive_to_bus(const ::foxglove_msgs::msg::CubePrimitive &msg) {
  ::foxglove_msgs::msg::v1::CubePrimitive bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  *bus.mutable_size() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.size);
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.color);
  return bus;
}

inline ::foxglove_msgs::msg::CubePrimitive cube_primitive_to_ros(const ::foxglove_msgs::msg::v1::CubePrimitive &bus) {
  ::foxglove_msgs::msg::CubePrimitive out;
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.size = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.size());
  out.color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.color());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsCubePrimitiveMapper
    : public TypedTopicMapper<FoxgloveMsgsCubePrimitiveMapper, ::foxglove_msgs::msg::CubePrimitive> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::CubePrimitive &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::cube_primitive_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::CubePrimitive bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::CubePrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::cube_primitive_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsCubePrimitiveMapper : TopicMapper {};
#endif

}  // namespace robot_bus
