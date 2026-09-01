#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/color_rgba.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/color_rgba.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::ColorRGBA color_rgba_to_bus(const ::std_msgs::msg::ColorRGBA &msg) {
  ::std_msgs::msg::v1::ColorRGBA bus;
  bus.set_r(msg.r);
  bus.set_g(msg.g);
  bus.set_b(msg.b);
  bus.set_a(msg.a);
  return bus;
}

inline ::std_msgs::msg::ColorRGBA color_rgba_to_ros(const ::std_msgs::msg::v1::ColorRGBA &bus) {
  ::std_msgs::msg::ColorRGBA out;
  out.r = bus.r();
  out.g = bus.g();
  out.b = bus.b();
  out.a = bus.a();
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsColorRgbaMapper
    : public TypedTopicMapper<StdMsgsColorRgbaMapper, ::std_msgs::msg::ColorRGBA> {
 public:
  const char *type_name() const override { return "std_msgs/msg/ColorRGBA"; }

  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::ColorRGBA &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::color_rgba_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::ColorRGBA bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::ColorRGBA bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::color_rgba_to_ros(bus);
  }
};
#else
struct StdMsgsColorRgbaMapper : TopicMapper {
  const char *type_name() const override { return "std_msgs/msg/ColorRGBA"; }
};
#endif

}  // namespace robot_bus
