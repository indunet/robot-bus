#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/diagnostic_msgs/msg/v1/key_value.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <diagnostic_msgs/msg/key_value.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace diagnostic_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::diagnostic_msgs::msg::v1::KeyValue key_value_to_bus(const ::diagnostic_msgs::msg::KeyValue &msg) {
  ::diagnostic_msgs::msg::v1::KeyValue bus;
  bus.set_key(msg.key.c_str());
  bus.set_value(msg.value.c_str());
  return bus;
}

inline ::diagnostic_msgs::msg::KeyValue key_value_to_ros(const ::diagnostic_msgs::msg::v1::KeyValue &bus) {
  ::diagnostic_msgs::msg::KeyValue out;
  out.key = bus.key();
  out.value = bus.value();
  return out;
}
#endif

}  // namespace diagnostic_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class DiagnosticMsgsKeyValueMapper
    : public TypedTopicMapper<DiagnosticMsgsKeyValueMapper, ::diagnostic_msgs::msg::KeyValue> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::diagnostic_msgs::msg::KeyValue &msg) const {
    auto bus = ros2_bridge_mappers::diagnostic_msgs::key_value_to_bus(msg);
    return encode_pb(bus);
  }

  ::diagnostic_msgs::msg::KeyValue bus_to_ros(BytesView payload) const {
    ::diagnostic_msgs::msg::v1::KeyValue bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::diagnostic_msgs::key_value_to_ros(bus);
  }
};
#else
struct DiagnosticMsgsKeyValueMapper : TopicMapper {};
#endif

}  // namespace robot_bus
