#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/multi_array_msgs.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/multi_array_layout.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/int16_multi_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::Int16MultiArray int16_multi_array_to_bus(const ::std_msgs::msg::Int16MultiArray &msg) {
  ::std_msgs::msg::v1::Int16MultiArray bus;
  *bus.mutable_layout() = ::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_layout_to_bus(msg.layout);
  for (auto x : msg.data) {
    bus.add_data(x);
  }
  return bus;
}

inline ::std_msgs::msg::Int16MultiArray int16_multi_array_to_ros(const ::std_msgs::msg::v1::Int16MultiArray &bus) {
  ::std_msgs::msg::Int16MultiArray out;
  out.layout = ::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_layout_to_ros(bus.layout());
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsInt16MultiArrayMapper
    : public TypedTopicMapper<StdMsgsInt16MultiArrayMapper, ::std_msgs::msg::Int16MultiArray> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::Int16MultiArray &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::int16_multi_array_to_bus(msg);
    return encode_pb(bus);
  }

  ::std_msgs::msg::Int16MultiArray bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::Int16MultiArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::int16_multi_array_to_ros(bus);
  }
};
#else
struct StdMsgsInt16MultiArrayMapper : TopicMapper {};
#endif

}  // namespace robot_bus
