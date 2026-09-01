#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/laser_scan.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/laser_scan.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::LaserScan laser_scan_to_bus(const ::foxglove_msgs::msg::LaserScan &msg) {
  ::foxglove_msgs::msg::v1::LaserScan bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_bus(msg.pose);
  bus.set_start_angle(msg.start_angle);
  bus.set_end_angle(msg.end_angle);
  for (auto x : msg.ranges) {
    bus.add_ranges(x);
  }
  for (auto x : msg.intensities) {
    bus.add_intensities(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::LaserScan laser_scan_to_ros(const ::foxglove_msgs::msg::v1::LaserScan &bus) {
  ::foxglove_msgs::msg::LaserScan out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.pose = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::pose_to_ros(bus.pose());
  out.start_angle = bus.start_angle();
  out.end_angle = bus.end_angle();
  out.ranges.assign(bus.ranges().begin(), bus.ranges().end());
  out.intensities.assign(bus.intensities().begin(), bus.intensities().end());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsLaserScanMapper
    : public TypedTopicMapper<FoxgloveMsgsLaserScanMapper, ::foxglove_msgs::msg::LaserScan> {
 public:
  const char *type_name() const override { return "foxglove_msgs/msg/LaserScan"; }

  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::LaserScan &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::laser_scan_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::LaserScan bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::LaserScan bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::laser_scan_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsLaserScanMapper : TopicMapper {
  const char *type_name() const override { return "foxglove_msgs/msg/LaserScan"; }
};
#endif

}  // namespace robot_bus
