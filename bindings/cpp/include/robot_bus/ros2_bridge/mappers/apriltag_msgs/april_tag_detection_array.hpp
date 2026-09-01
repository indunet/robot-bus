#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/apriltag_msgs/msg/v1/april_tag_detection_array.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/apriltag_msgs/april_tag_detection.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
#include <apriltag_msgs/msg/april_tag_detection_array.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace apriltag_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
inline ::apriltag_msgs::msg::v1::AprilTagDetectionArray april_tag_detection_array_to_bus(const ::apriltag_msgs::msg::AprilTagDetectionArray &msg) {
  ::apriltag_msgs::msg::v1::AprilTagDetectionArray bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  for (const auto &x : msg.detections) {
    *bus.add_detections() = ::robot_bus::ros2_bridge_mappers::apriltag_msgs::april_tag_detection_to_bus(x);
  }
  return bus;
}

inline ::apriltag_msgs::msg::AprilTagDetectionArray april_tag_detection_array_to_ros(const ::apriltag_msgs::msg::v1::AprilTagDetectionArray &bus) {
  ::apriltag_msgs::msg::AprilTagDetectionArray out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.detections.clear();
  for (const auto &x : bus.detections()) {
    out.detections.push_back(::robot_bus::ros2_bridge_mappers::apriltag_msgs::april_tag_detection_to_ros(x));
  }
  return out;
}
#endif

}  // namespace apriltag_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
class ApriltagMsgsAprilTagDetectionArrayMapper
    : public TypedTopicMapper<ApriltagMsgsAprilTagDetectionArrayMapper, ::apriltag_msgs::msg::AprilTagDetectionArray> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::apriltag_msgs::msg::AprilTagDetectionArray &msg) const {
    auto bus = ros2_bridge_mappers::apriltag_msgs::april_tag_detection_array_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::apriltag_msgs::msg::AprilTagDetectionArray bus_to_ros(BytesView payload) const {
    ::apriltag_msgs::msg::v1::AprilTagDetectionArray bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::apriltag_msgs::april_tag_detection_array_to_ros(bus);
  }
};
#else
struct ApriltagMsgsAprilTagDetectionArrayMapper : TopicMapper {};
#endif

}  // namespace robot_bus
