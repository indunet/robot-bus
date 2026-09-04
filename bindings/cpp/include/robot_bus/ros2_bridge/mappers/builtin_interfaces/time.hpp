#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/builtin_interfaces/msg/v1/time.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <builtin_interfaces/msg/time.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace builtin_interfaces {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::builtin_interfaces::msg::v1::Time time_to_bus(const ::builtin_interfaces::msg::Time &msg) {
  ::builtin_interfaces::msg::v1::Time bus;
  bus.set_sec(msg.sec);
  bus.set_nanosec(msg.nanosec);
  return bus;
}

inline ::builtin_interfaces::msg::Time time_to_ros(const ::builtin_interfaces::msg::v1::Time &bus) {
  ::builtin_interfaces::msg::Time out;
  out.sec = bus.sec();
  out.nanosec = bus.nanosec();
  return out;
}
#endif

}  // namespace builtin_interfaces
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class BuiltinInterfacesTimeMapper
    : public TypedTopicMapper<BuiltinInterfacesTimeMapper, ::builtin_interfaces::msg::Time> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::builtin_interfaces::msg::Time &msg) const {
    auto bus = ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg);
    return encode_pb(bus);
  }

  ::builtin_interfaces::msg::Time bus_to_ros(BytesView payload) const {
    ::builtin_interfaces::msg::v1::Time bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus);
  }
};
#else
struct BuiltinInterfacesTimeMapper : TopicMapper {};
#endif

}  // namespace robot_bus
