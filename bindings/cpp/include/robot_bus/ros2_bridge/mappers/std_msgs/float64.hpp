#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/primitives.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/float64.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::Float64 float64_to_bus(const ::std_msgs::msg::Float64 &msg) {
  ::std_msgs::msg::v1::Float64 bus;
  bus.set_data(msg.data);
  return bus;
}

inline ::std_msgs::msg::Float64 float64_to_ros(const ::std_msgs::msg::v1::Float64 &bus) {
  ::std_msgs::msg::Float64 out;
  out.data = bus.data();
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsFloat64Mapper
    : public TypedTopicMapper<StdMsgsFloat64Mapper, ::std_msgs::msg::Float64> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::Float64 &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::float64_to_bus(msg);
    return encode_pb(bus);
  }

  ::std_msgs::msg::Float64 bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::Float64 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::float64_to_ros(bus);
  }
};
#else
struct StdMsgsFloat64Mapper : TopicMapper {};
#endif

}  // namespace robot_bus
