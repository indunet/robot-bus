#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/multi_array_msgs.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/multi_array_layout.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/float64_multi_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::Float64MultiArray float64_multi_array_to_bus(const ::std_msgs::msg::Float64MultiArray &msg) {
  ::std_msgs::msg::v1::Float64MultiArray bus;
  *bus.mutable_layout() = ::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_layout_to_bus(msg.layout);
  for (auto x : msg.data) {
    bus.add_data(x);
  }
  return bus;
}

inline ::std_msgs::msg::Float64MultiArray float64_multi_array_to_ros(const ::std_msgs::msg::v1::Float64MultiArray &bus) {
  ::std_msgs::msg::Float64MultiArray out;
  out.layout = ::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_layout_to_ros(bus.layout());
  ::robot_bus::ros2_bridge_mappers::copy_seq(out.data, bus.data());
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsFloat64MultiArrayMapper
    : public TypedTopicMapper<StdMsgsFloat64MultiArrayMapper, ::std_msgs::msg::Float64MultiArray> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::Float64MultiArray &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::float64_multi_array_to_bus(msg);
    return encode_pb(bus);
  }

  ::std_msgs::msg::Float64MultiArray bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::Float64MultiArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::float64_multi_array_to_ros(bus);
  }
};
#else
struct StdMsgsFloat64MultiArrayMapper : TopicMapper {};
#endif

}  // namespace robot_bus
