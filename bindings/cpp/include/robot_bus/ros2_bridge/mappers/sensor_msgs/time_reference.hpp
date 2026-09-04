#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/time_reference.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/builtin_interfaces/time.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/time_reference.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::TimeReference time_reference_to_bus(const ::sensor_msgs::msg::TimeReference &msg) {
  ::sensor_msgs::msg::v1::TimeReference bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_time_ref() = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_bus(msg.time_ref);
  bus.set_source(msg.source.c_str());
  return bus;
}

inline ::sensor_msgs::msg::TimeReference time_reference_to_ros(const ::sensor_msgs::msg::v1::TimeReference &bus) {
  ::sensor_msgs::msg::TimeReference out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.time_ref = ::robot_bus::ros2_bridge_mappers::builtin_interfaces::time_to_ros(bus.time_ref());
  out.source = bus.source();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsTimeReferenceMapper
    : public TypedTopicMapper<SensorMsgsTimeReferenceMapper, ::sensor_msgs::msg::TimeReference> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::TimeReference &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::time_reference_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::TimeReference bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::TimeReference bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::time_reference_to_ros(bus);
  }
};
#else
struct SensorMsgsTimeReferenceMapper : TopicMapper {};
#endif

}  // namespace robot_bus
