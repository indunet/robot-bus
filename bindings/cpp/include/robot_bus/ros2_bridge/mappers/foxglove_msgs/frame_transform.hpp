#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/frame_transform.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/vector3.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/quaternion.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/frame_transform.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::FrameTransform frame_transform_to_bus(const ::foxglove_msgs::msg::FrameTransform &msg) {
  ::foxglove_msgs::msg::v1::FrameTransform bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_parent_frame_id(msg.parent_frame_id.c_str());
  bus.set_child_frame_id(msg.child_frame_id.c_str());
  *bus.mutable_translation() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_bus(msg.translation);
  *bus.mutable_rotation() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::quaternion_to_bus(msg.rotation);
  return bus;
}

inline ::foxglove_msgs::msg::FrameTransform frame_transform_to_ros(const ::foxglove_msgs::msg::v1::FrameTransform &bus) {
  ::foxglove_msgs::msg::FrameTransform out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.parent_frame_id = bus.parent_frame_id();
  out.child_frame_id = bus.child_frame_id();
  out.translation = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::vector3_to_ros(bus.translation());
  out.rotation = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::quaternion_to_ros(bus.rotation());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsFrameTransformMapper
    : public TypedTopicMapper<FoxgloveMsgsFrameTransformMapper, ::foxglove_msgs::msg::FrameTransform> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/FrameTransform"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::FrameTransform &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::frame_transform_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::FrameTransform bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::FrameTransform bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::frame_transform_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsFrameTransformMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/FrameTransform"; }
};
#endif

}  // namespace robot_bus
