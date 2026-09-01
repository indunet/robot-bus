#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/apriltag_msgs/msg/v1/april_tag_detection.pb.h>
#include <robot_bus/ros2_bridge/mappers/apriltag_msgs/point.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
#include <apriltag_msgs/msg/april_tag_detection.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace apriltag_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
inline ::apriltag_msgs::msg::v1::AprilTagDetection april_tag_detection_to_bus(const ::apriltag_msgs::msg::AprilTagDetection &msg) {
  ::apriltag_msgs::msg::v1::AprilTagDetection bus;
  bus.set_family(msg.family.c_str());
  bus.set_id(msg.id);
  bus.set_hamming(msg.hamming);
  bus.set_goodness(msg.goodness);
  bus.set_decision_margin(msg.decision_margin);
  *bus.mutable_centre() = ::robot_bus::ros2_bridge_mappers::apriltag_msgs::point_to_bus(msg.centre);
  for (const auto &x : msg.corners) {
    *bus.add_corners() = ::robot_bus::ros2_bridge_mappers::apriltag_msgs::point_to_bus(x);
  }
  for (auto x : msg.homography) {
    bus.add_homography(x);
  }
  return bus;
}

inline ::apriltag_msgs::msg::AprilTagDetection april_tag_detection_to_ros(const ::apriltag_msgs::msg::v1::AprilTagDetection &bus) {
  ::apriltag_msgs::msg::AprilTagDetection out;
  out.family = bus.family();
  out.id = bus.id();
  out.hamming = bus.hamming();
  out.goodness = bus.goodness();
  out.decision_margin = bus.decision_margin();
  out.centre = ::robot_bus::ros2_bridge_mappers::apriltag_msgs::point_to_ros(bus.centre());
  out.corners.clear();
  for (const auto &x : bus.corners()) {
    out.corners.push_back(::robot_bus::ros2_bridge_mappers::apriltag_msgs::point_to_ros(x));
  }
  out.homography.assign(bus.homography().begin(), bus.homography().end());
  return out;
}
#endif

}  // namespace apriltag_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
class ApriltagMsgsAprilTagDetectionMapper
    : public TypedTopicMapper<ApriltagMsgsAprilTagDetectionMapper, ::apriltag_msgs::msg::AprilTagDetection> {
 public:
  const char *type_name() const override { return "apriltag_msgs/msg/AprilTagDetection"; }

  std::vector<uint8_t> ros_to_bus(const ::apriltag_msgs::msg::AprilTagDetection &msg) const {
    auto bus = ros2_bridge_mappers::apriltag_msgs::april_tag_detection_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::apriltag_msgs::msg::AprilTagDetection bus_to_ros(BytesView payload) const {
    ::apriltag_msgs::msg::v1::AprilTagDetection bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::apriltag_msgs::april_tag_detection_to_ros(bus);
  }
};
#else
struct ApriltagMsgsAprilTagDetectionMapper : TopicMapper {
  const char *type_name() const override { return "apriltag_msgs/msg/AprilTagDetection"; }
};
#endif

}  // namespace robot_bus
