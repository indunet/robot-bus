#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/color.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/color.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Color color_to_bus(const ::foxglove_msgs::msg::Color &msg) {
  ::foxglove_msgs::msg::v1::Color bus;
  bus.set_r(msg.r);
  bus.set_g(msg.g);
  bus.set_b(msg.b);
  bus.set_a(msg.a);
  return bus;
}

inline ::foxglove_msgs::msg::Color color_to_ros(const ::foxglove_msgs::msg::v1::Color &bus) {
  ::foxglove_msgs::msg::Color out;
  out.r = bus.r();
  out.g = bus.g();
  out.b = bus.b();
  out.a = bus.a();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsColorMapper
    : public TypedTopicMapper<FoxgloveMsgsColorMapper, ::foxglove_msgs::msg::Color> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/Color"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Color &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::color_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Color bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Color bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::color_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsColorMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/Color"; }
};
#endif

}  // namespace robot_bus
