#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/multi_echo_laser_scan.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/laser_echo.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/multi_echo_laser_scan.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::MultiEchoLaserScan multi_echo_laser_scan_to_bus(const ::sensor_msgs::msg::MultiEchoLaserScan &msg) {
  ::sensor_msgs::msg::v1::MultiEchoLaserScan bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_angle_min(msg.angle_min);
  bus.set_angle_max(msg.angle_max);
  bus.set_angle_increment(msg.angle_increment);
  bus.set_time_increment(msg.time_increment);
  bus.set_scan_time(msg.scan_time);
  bus.set_range_min(msg.range_min);
  bus.set_range_max(msg.range_max);
  for (const auto &x : msg.ranges) {
    *bus.add_ranges() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::laser_echo_to_bus(x);
  }
  for (const auto &x : msg.intensities) {
    *bus.add_intensities() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::laser_echo_to_bus(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::MultiEchoLaserScan multi_echo_laser_scan_to_ros(const ::sensor_msgs::msg::v1::MultiEchoLaserScan &bus) {
  ::sensor_msgs::msg::MultiEchoLaserScan out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.angle_min = bus.angle_min();
  out.angle_max = bus.angle_max();
  out.angle_increment = bus.angle_increment();
  out.time_increment = bus.time_increment();
  out.scan_time = bus.scan_time();
  out.range_min = bus.range_min();
  out.range_max = bus.range_max();
  out.ranges.clear();
  for (const auto &x : bus.ranges()) {
    out.ranges.push_back(::robot_bus::ros2_bridge_mappers::sensor_msgs::laser_echo_to_ros(x));
  }
  out.intensities.clear();
  for (const auto &x : bus.intensities()) {
    out.intensities.push_back(::robot_bus::ros2_bridge_mappers::sensor_msgs::laser_echo_to_ros(x));
  }
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsMultiEchoLaserScanMapper
    : public TypedTopicMapper<SensorMsgsMultiEchoLaserScanMapper, ::sensor_msgs::msg::MultiEchoLaserScan> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::MultiEchoLaserScan &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::multi_echo_laser_scan_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::MultiEchoLaserScan bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::MultiEchoLaserScan bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::multi_echo_laser_scan_to_ros(bus);
  }
};
#else
struct SensorMsgsMultiEchoLaserScanMapper : TopicMapper {};
#endif

}  // namespace robot_bus
