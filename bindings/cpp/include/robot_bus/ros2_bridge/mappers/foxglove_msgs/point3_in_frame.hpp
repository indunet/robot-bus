#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/point3_in_frame.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/point3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/point3_in_frame.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::Point3InFrame point3_in_frame_to_bus(const ::foxglove_msgs::msg::Point3InFrame &msg) {
  ::foxglove_msgs::msg::v1::Point3InFrame bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  *bus.mutable_point() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::point3_to_bus(msg.point);
  return bus;
}

inline ::foxglove_msgs::msg::Point3InFrame point3_in_frame_to_ros(const ::foxglove_msgs::msg::v1::Point3InFrame &bus) {
  ::foxglove_msgs::msg::Point3InFrame out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.point = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::point3_to_ros(bus.point());
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPoint3InFrameMapper
    : public TypedTopicMapper<FoxgloveMsgsPoint3InFrameMapper, ::foxglove_msgs::msg::Point3InFrame> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::Point3InFrame &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::point3_in_frame_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::Point3InFrame bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::Point3InFrame bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::point3_in_frame_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPoint3InFrameMapper : TopicMapper {};
#endif

}  // namespace robot_bus
