#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/primitives.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/int8.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::Int8 int8_to_bus(const ::std_msgs::msg::Int8 &msg) {
  ::std_msgs::msg::v1::Int8 bus;
  bus.set_data(static_cast<int32_t>(msg.data));
  return bus;
}

inline ::std_msgs::msg::Int8 int8_to_ros(const ::std_msgs::msg::v1::Int8 &bus) {
  ::std_msgs::msg::Int8 out;
  out.data = static_cast<int8_t>(bus.data());
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsInt8Mapper
    : public TypedTopicMapper<StdMsgsInt8Mapper, ::std_msgs::msg::Int8> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::Int8 &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::int8_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::Int8 bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::Int8 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::int8_to_ros(bus);
  }
};
#else
struct StdMsgsInt8Mapper : TopicMapper {};
#endif

}  // namespace robot_bus
