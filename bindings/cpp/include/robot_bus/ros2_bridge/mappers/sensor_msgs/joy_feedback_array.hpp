#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/joy_feedback_array.pb.h>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/joy_feedback.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/joy_feedback_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::JoyFeedbackArray joy_feedback_array_to_bus(const ::sensor_msgs::msg::JoyFeedbackArray &msg) {
  ::sensor_msgs::msg::v1::JoyFeedbackArray bus;
  for (const auto &x : msg.array) {
    *bus.add_array() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::joy_feedback_to_bus(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::JoyFeedbackArray joy_feedback_array_to_ros(const ::sensor_msgs::msg::v1::JoyFeedbackArray &bus) {
  ::sensor_msgs::msg::JoyFeedbackArray out;
  out.array.clear();
  for (const auto &x : bus.array()) {
    out.array.push_back(::robot_bus::ros2_bridge_mappers::sensor_msgs::joy_feedback_to_ros(x));
  }
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsJoyFeedbackArrayMapper
    : public TypedTopicMapper<SensorMsgsJoyFeedbackArrayMapper, ::sensor_msgs::msg::JoyFeedbackArray> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::JoyFeedbackArray &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::joy_feedback_array_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::JoyFeedbackArray bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::JoyFeedbackArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::joy_feedback_array_to_ros(bus);
  }
};
#else
struct SensorMsgsJoyFeedbackArrayMapper : TopicMapper {};
#endif

}  // namespace robot_bus
