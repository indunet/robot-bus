#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/cylinder_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/color.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/cylinder_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::CylinderPrimitive cylinder_primitive_to_bus(const ::foxglove_msgs::msg::CylinderPrimitive &msg) {
  ::foxglove_msgs::msg::v1::CylinderPrimitive bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  *bus.mutable_size() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.size);
  bus.set_bottom_scale(msg.bottom_scale);
  bus.set_top_scale(msg.top_scale);
  *bus.mutable_color() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg.color);
  return bus;
}

inline ::foxglove_msgs::msg::CylinderPrimitive cylinder_primitive_to_ros(const ::foxglove_msgs::msg::v1::CylinderPrimitive &bus) {
  ::foxglove_msgs::msg::CylinderPrimitive out;
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.size = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.size());
  out.bottom_scale = bus.bottom_scale();
  out.top_scale = bus.top_scale();
  out.color = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus.color());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsCylinderPrimitiveMapper
    : public TypedTopicMapper<FoxgloveMsgsCylinderPrimitiveMapper, ::foxglove_msgs::msg::CylinderPrimitive> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/CylinderPrimitive"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::CylinderPrimitive &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::cylinder_primitive_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::CylinderPrimitive bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::CylinderPrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::cylinder_primitive_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsCylinderPrimitiveMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/CylinderPrimitive"; }
};
#endif

}  // namespace robot_bus
