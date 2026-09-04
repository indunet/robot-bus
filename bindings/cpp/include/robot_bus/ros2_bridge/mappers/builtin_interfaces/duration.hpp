#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/builtin_interfaces/msg/v1/duration.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <builtin_interfaces/msg/duration.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace builtin_interfaces {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::builtin_interfaces::msg::v1::Duration duration_to_bus(const ::builtin_interfaces::msg::Duration &msg) {
  ::builtin_interfaces::msg::v1::Duration bus;
  bus.set_sec(msg.sec);
  bus.set_nanosec(msg.nanosec);
  return bus;
}

inline ::builtin_interfaces::msg::Duration duration_to_ros(const ::builtin_interfaces::msg::v1::Duration &bus) {
  ::builtin_interfaces::msg::Duration out;
  out.sec = bus.sec();
  out.nanosec = bus.nanosec();
  return out;
}
#endif

}  // namespace builtin_interfaces
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class BuiltinInterfacesDurationMapper
    : public TypedTopicMapper<BuiltinInterfacesDurationMapper, ::builtin_interfaces::msg::Duration> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::builtin_interfaces::msg::Duration &msg) const {
    auto bus = ros2_bridge_mappers::builtin_interfaces::duration_to_bus(msg);
    return encode_pb(bus);
  }

  ::builtin_interfaces::msg::Duration bus_to_ros(BytesView payload) const {
    ::builtin_interfaces::msg::v1::Duration bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::builtin_interfaces::duration_to_ros(bus);
  }
};
#else
struct BuiltinInterfacesDurationMapper : TopicMapper {};
#endif

}  // namespace robot_bus
