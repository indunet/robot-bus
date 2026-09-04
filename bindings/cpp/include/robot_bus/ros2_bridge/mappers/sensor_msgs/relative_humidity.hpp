#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/relative_humidity.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/relative_humidity.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::RelativeHumidity relative_humidity_to_bus(const ::sensor_msgs::msg::RelativeHumidity &msg) {
  ::sensor_msgs::msg::v1::RelativeHumidity bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_relative_humidity(msg.relative_humidity);
  bus.set_variance(msg.variance);
  return bus;
}

inline ::sensor_msgs::msg::RelativeHumidity relative_humidity_to_ros(const ::sensor_msgs::msg::v1::RelativeHumidity &bus) {
  ::sensor_msgs::msg::RelativeHumidity out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.relative_humidity = bus.relative_humidity();
  out.variance = bus.variance();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsRelativeHumidityMapper
    : public TypedTopicMapper<SensorMsgsRelativeHumidityMapper, ::sensor_msgs::msg::RelativeHumidity> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::RelativeHumidity &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::relative_humidity_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::RelativeHumidity bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::RelativeHumidity bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::relative_humidity_to_ros(bus);
  }
};
#else
struct SensorMsgsRelativeHumidityMapper : TopicMapper {};
#endif

}  // namespace robot_bus
