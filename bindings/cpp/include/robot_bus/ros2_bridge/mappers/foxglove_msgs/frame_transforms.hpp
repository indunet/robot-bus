#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/frame_transforms.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/frame_transform.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/frame_transforms.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::FrameTransforms frame_transforms_to_bus(const ::foxglove_msgs::msg::FrameTransforms &msg) {
  ::foxglove_msgs::msg::v1::FrameTransforms bus;
  for (const auto &x : msg.transforms) {
    *bus.add_transforms() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::frame_transform_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::FrameTransforms frame_transforms_to_ros(const ::foxglove_msgs::msg::v1::FrameTransforms &bus) {
  ::foxglove_msgs::msg::FrameTransforms out;
  out.transforms.clear();
  for (const auto &x : bus.transforms()) {
    out.transforms.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::frame_transform_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsFrameTransformsMapper
    : public TypedTopicMapper<FoxgloveMsgsFrameTransformsMapper, ::foxglove_msgs::msg::FrameTransforms> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::FrameTransforms &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::frame_transforms_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::FrameTransforms bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::FrameTransforms bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::frame_transforms_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsFrameTransformsMapper : TopicMapper {};
#endif

}  // namespace robot_bus
