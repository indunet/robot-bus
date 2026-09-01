#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/multi_array.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/multi_array_dimension.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/multi_array_layout.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::MultiArrayLayout multi_array_layout_to_bus(const ::std_msgs::msg::MultiArrayLayout &msg) {
  ::std_msgs::msg::v1::MultiArrayLayout bus;
  for (const auto &x : msg.dim) {
    *bus.add_dim() = ::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_dimension_to_bus(x);
  }
  bus.set_data_offset(msg.data_offset);
  return bus;
}

inline ::std_msgs::msg::MultiArrayLayout multi_array_layout_to_ros(const ::std_msgs::msg::v1::MultiArrayLayout &bus) {
  ::std_msgs::msg::MultiArrayLayout out;
  out.dim.clear();
  for (const auto &x : bus.dim()) {
    out.dim.push_back(::robot_bus::ros2_bridge_mappers::std_msgs::multi_array_dimension_to_ros(x));
  }
  out.data_offset = bus.data_offset();
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsMultiArrayLayoutMapper
    : public TypedTopicMapper<StdMsgsMultiArrayLayoutMapper, ::std_msgs::msg::MultiArrayLayout> {
 public:
  const char *type_name() const override { return "std_msgs/msg/MultiArrayLayout"; }

  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::MultiArrayLayout &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::multi_array_layout_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::MultiArrayLayout bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::MultiArrayLayout bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::multi_array_layout_to_ros(bus);
  }
};
#else
struct StdMsgsMultiArrayLayoutMapper : TopicMapper {
  const char *type_name() const override { return "std_msgs/msg/MultiArrayLayout"; }
};
#endif

}  // namespace robot_bus
