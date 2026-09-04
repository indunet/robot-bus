#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/joy.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/joy.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::Joy joy_to_bus(const ::sensor_msgs::msg::Joy &msg) {
  ::sensor_msgs::msg::v1::Joy bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (auto x : msg.axes) {
    bus.add_axes(x);
  }
  for (auto x : msg.buttons) {
    bus.add_buttons(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::Joy joy_to_ros(const ::sensor_msgs::msg::v1::Joy &bus) {
  ::sensor_msgs::msg::Joy out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.axes.assign(bus.axes().begin(), bus.axes().end());
  out.buttons.assign(bus.buttons().begin(), bus.buttons().end());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsJoyMapper
    : public TypedTopicMapper<SensorMsgsJoyMapper, ::sensor_msgs::msg::Joy> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::Joy &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::joy_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::Joy bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::Joy bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::joy_to_ros(bus);
  }
};
#else
struct SensorMsgsJoyMapper : TopicMapper {};
#endif

}  // namespace robot_bus
