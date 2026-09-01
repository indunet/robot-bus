#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/pose.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/quaternion.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/pose.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Pose pose_to_bus(const ::foxglove_msgs::msg::Pose &msg) {
  ::foxglove_msgs::msg::v1::Pose bus;
  *bus.mutable_position() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.position);
  *bus.mutable_orientation() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::quaternion_to_bus(msg.orientation);
  return bus;
}

inline ::foxglove_msgs::msg::Pose pose_to_ros(const ::foxglove_msgs::msg::v1::Pose &bus) {
  ::foxglove_msgs::msg::Pose out;
  out.position = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.position());
  out.orientation = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::quaternion_to_ros(bus.orientation());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPoseMapper
    : public TypedTopicMapper<FoxgloveMsgsPoseMapper, ::foxglove_msgs::msg::Pose> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/Pose"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Pose &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Pose bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Pose bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPoseMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/Pose"; }
};
#endif

}  // namespace robot_bus
