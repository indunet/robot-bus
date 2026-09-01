#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/multi_array_msgs.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/multi_array_layout.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/byte_multi_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::ByteMultiArray byte_multi_array_to_bus(const ::std_msgs::msg::ByteMultiArray &msg) {
  ::std_msgs::msg::v1::ByteMultiArray bus;
  *bus.mutable_layout() = ::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_layout_to_bus(msg.layout);
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::std_msgs::msg::ByteMultiArray byte_multi_array_to_ros(const ::std_msgs::msg::v1::ByteMultiArray &bus) {
  ::std_msgs::msg::ByteMultiArray out;
  out.layout = ::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_layout_to_ros(bus.layout());
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsByteMultiArrayMapper
    : public TypedTopicMapper<StdMsgsByteMultiArrayMapper, ::std_msgs::msg::ByteMultiArray> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::ByteMultiArray &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::byte_multi_array_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::ByteMultiArray bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::ByteMultiArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::byte_multi_array_to_ros(bus);
  }
};
#else
struct StdMsgsByteMultiArrayMapper : TopicMapper {};
#endif

}  // namespace robot_bus
