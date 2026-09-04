#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/tf2_msgs/msg/v1/tf_message.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/transform_stamped.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <tf2_msgs/msg/tf_message.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace tf2_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::tf2_msgs::msg::v1::TFMessage tf_message_to_bus(const ::tf2_msgs::msg::TFMessage &msg) {
  ::tf2_msgs::msg::v1::TFMessage bus;
  for (const auto &x : msg.transforms) {
    *bus.add_transforms() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_bus(x);
  }
  return bus;
}

inline ::tf2_msgs::msg::TFMessage tf_message_to_ros(const ::tf2_msgs::msg::v1::TFMessage &bus) {
  ::tf2_msgs::msg::TFMessage out;
  out.transforms.clear();
  for (const auto &x : bus.transforms()) {
    out.transforms.push_back(::robot_bus::ros2_bridge_mappers::geometry_msgs::transform_stamped_to_ros(x));
  }
  return out;
}
#endif

}  // namespace tf2_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class Tf2MsgsTfMessageMapper
    : public TypedTopicMapper<Tf2MsgsTfMessageMapper, ::tf2_msgs::msg::TFMessage> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::tf2_msgs::msg::TFMessage &msg) const {
    auto bus = ros2_bridge_mappers::tf2_msgs::tf_message_to_bus(msg);
    return encode_pb(bus);
  }

  ::tf2_msgs::msg::TFMessage bus_to_ros(BytesView payload) const {
    ::tf2_msgs::msg::v1::TFMessage bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::tf2_msgs::tf_message_to_ros(bus);
  }
};
#else
struct Tf2MsgsTfMessageMapper : TopicMapper {};
#endif

}  // namespace robot_bus
