#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/std_msgs/msg/v1/multi_array.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <std_msgs/msg/multi_array_dimension.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace std_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::std_msgs::msg::v1::MultiArrayDimension multi_array_dimension_to_bus(const ::std_msgs::msg::MultiArrayDimension &msg) {
  ::std_msgs::msg::v1::MultiArrayDimension bus;
  bus.set_label(msg.label.c_str());
  bus.set_size(msg.size);
  bus.set_stride(msg.stride);
  return bus;
}

inline ::std_msgs::msg::MultiArrayDimension multi_array_dimension_to_ros(const ::std_msgs::msg::v1::MultiArrayDimension &bus) {
  ::std_msgs::msg::MultiArrayDimension out;
  out.label = bus.label();
  out.size = bus.size();
  out.stride = bus.stride();
  return out;
}
#endif

}  // namespace std_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class StdMsgsMultiArrayDimensionMapper
    : public TypedTopicMapper<StdMsgsMultiArrayDimensionMapper, ::std_msgs::msg::MultiArrayDimension> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::std_msgs::msg::MultiArrayDimension &msg) const {
    auto bus = ros2_bridge_mappers::std_msgs::multi_array_dimension_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::std_msgs::msg::MultiArrayDimension bus_to_ros(BytesView payload) const {
    ::std_msgs::msg::v1::MultiArrayDimension bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::std_msgs::multi_array_dimension_to_ros(bus);
  }
};
#else
struct StdMsgsMultiArrayDimensionMapper : TopicMapper {};
#endif

}  // namespace robot_bus
