#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/arrow_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/arrow_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::ArrowPrimitive arrow_primitive_to_bus(const ::foxglove_msgs::msg::ArrowPrimitive &msg) {
  ::foxglove_msgs::msg::v1::ArrowPrimitive bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  bus.set_shaft_length(msg.shaft_length);
  bus.set_shaft_diameter(msg.shaft_diameter);
  bus.set_head_length(msg.head_length);
  bus.set_head_diameter(msg.head_diameter);
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.color);
  return bus;
}

inline ::foxglove_msgs::msg::ArrowPrimitive arrow_primitive_to_ros(const ::foxglove_msgs::msg::v1::ArrowPrimitive &bus) {
  ::foxglove_msgs::msg::ArrowPrimitive out;
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.shaft_length = bus.shaft_length();
  out.shaft_diameter = bus.shaft_diameter();
  out.head_length = bus.head_length();
  out.head_diameter = bus.head_diameter();
  out.color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.color());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsArrowPrimitiveMapper
    : public TypedTopicMapper<FoxgloveMsgsArrowPrimitiveMapper, ::foxglove_msgs::msg::ArrowPrimitive> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/ArrowPrimitive"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::ArrowPrimitive &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::arrow_primitive_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::ArrowPrimitive bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::ArrowPrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::arrow_primitive_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsArrowPrimitiveMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/ArrowPrimitive"; }
};
#endif

}  // namespace robot_bus
