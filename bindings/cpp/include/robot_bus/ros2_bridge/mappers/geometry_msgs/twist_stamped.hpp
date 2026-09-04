#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/stamped.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/twist_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::TwistStamped twist_stamped_to_bus(const ::geometry_msgs::msg::TwistStamped &msg) {
  ::geometry_msgs::msg::v1::TwistStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_twist() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(msg.twist);
  return bus;
}

inline ::geometry_msgs::msg::TwistStamped twist_stamped_to_ros(const ::geometry_msgs::msg::v1::TwistStamped &bus) {
  ::geometry_msgs::msg::TwistStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.twist = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(bus.twist());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsTwistStampedMapper
    : public TypedTopicMapper<GeometryMsgsTwistStampedMapper, ::geometry_msgs::msg::TwistStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::TwistStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::twist_stamped_to_bus(msg);
    return encode_pb(bus);
  }

  ::geometry_msgs::msg::TwistStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::TwistStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::twist_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsTwistStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
