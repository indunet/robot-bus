#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/joy_feedback.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/joy_feedback.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::JoyFeedback joy_feedback_to_bus(const ::sensor_msgs::msg::JoyFeedback &msg) {
  ::sensor_msgs::msg::v1::JoyFeedback bus;
  bus.set_type(msg.type);
  bus.set_id(msg.id);
  bus.set_intensity(msg.intensity);
  return bus;
}

inline ::sensor_msgs::msg::JoyFeedback joy_feedback_to_ros(const ::sensor_msgs::msg::v1::JoyFeedback &bus) {
  ::sensor_msgs::msg::JoyFeedback out;
  out.type = bus.type();
  out.id = bus.id();
  out.intensity = bus.intensity();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsJoyFeedbackMapper
    : public TypedTopicMapper<SensorMsgsJoyFeedbackMapper, ::sensor_msgs::msg::JoyFeedback> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::JoyFeedback &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::joy_feedback_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::JoyFeedback bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::JoyFeedback bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::joy_feedback_to_ros(bus);
  }
};
#else
struct SensorMsgsJoyFeedbackMapper : TopicMapper {};
#endif

}  // namespace robot_bus
