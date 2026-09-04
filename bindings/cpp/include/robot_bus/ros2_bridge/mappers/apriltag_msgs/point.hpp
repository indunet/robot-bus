#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/apriltag_msgs/msg/v1/point.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
#include <apriltag_msgs/msg/point.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace apriltag_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
inline ::apriltag_msgs::msg::v1::Point point_to_bus(const ::apriltag_msgs::msg::Point &msg) {
  ::apriltag_msgs::msg::v1::Point bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  return bus;
}

inline ::apriltag_msgs::msg::Point point_to_ros(const ::apriltag_msgs::msg::v1::Point &bus) {
  ::apriltag_msgs::msg::Point out;
  out.x = bus.x();
  out.y = bus.y();
  return out;
}
#endif

}  // namespace apriltag_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_APRILTAG_MSGS)
class ApriltagMsgsPointMapper
    : public TypedTopicMapper<ApriltagMsgsPointMapper, ::apriltag_msgs::msg::Point> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::apriltag_msgs::msg::Point &msg) const {
    auto bus = ros2_bridge_mappers::apriltag_msgs::point_to_bus(msg);
    return encode_pb(bus);
  }

  ::apriltag_msgs::msg::Point bus_to_ros(BytesView payload) const {
    ::apriltag_msgs::msg::v1::Point bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::apriltag_msgs::point_to_ros(bus);
  }
};
#else
struct ApriltagMsgsPointMapper : TopicMapper {};
#endif

}  // namespace robot_bus
