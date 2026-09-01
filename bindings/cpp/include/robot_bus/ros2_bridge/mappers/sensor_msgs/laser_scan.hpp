#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/laser_scan.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/laser_scan.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::LaserScan laser_scan_to_bus(const ::sensor_msgs::msg::LaserScan &msg) {
  ::sensor_msgs::msg::v1::LaserScan bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_angle_min(msg.angle_min);
  bus.set_angle_max(msg.angle_max);
  bus.set_angle_increment(msg.angle_increment);
  bus.set_time_increment(msg.time_increment);
  bus.set_scan_time(msg.scan_time);
  bus.set_range_min(msg.range_min);
  bus.set_range_max(msg.range_max);
  for (auto x : msg.ranges) {
    bus.add_ranges(x);
  }
  for (auto x : msg.intensities) {
    bus.add_intensities(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::LaserScan laser_scan_to_ros(const ::sensor_msgs::msg::v1::LaserScan &bus) {
  ::sensor_msgs::msg::LaserScan out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.angle_min = bus.angle_min();
  out.angle_max = bus.angle_max();
  out.angle_increment = bus.angle_increment();
  out.time_increment = bus.time_increment();
  out.scan_time = bus.scan_time();
  out.range_min = bus.range_min();
  out.range_max = bus.range_max();
  out.ranges.assign(bus.ranges().begin(), bus.ranges().end());
  out.intensities.assign(bus.intensities().begin(), bus.intensities().end());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsLaserScanMapper
    : public TypedTopicMapper<SensorMsgsLaserScanMapper, ::sensor_msgs::msg::LaserScan> {
 public:
  const char *type_name() const override { return "sensor_msgs/msg/LaserScan"; }

  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::LaserScan &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::laser_scan_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::LaserScan bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::LaserScan bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::laser_scan_to_ros(bus);
  }
};
#else
struct SensorMsgsLaserScanMapper : TopicMapper {
  const char *type_name() const override { return "sensor_msgs/msg/LaserScan"; }
};
#endif

}  // namespace robot_bus
