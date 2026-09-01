#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/twist.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/twist.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Twist twist_to_bus(const ::geometry_msgs::msg::Twist &msg) {
  ::geometry_msgs::msg::v1::Twist bus;
  *bus.mutable_linear() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.linear);
  *bus.mutable_angular() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.angular);
  return bus;
}

inline ::geometry_msgs::msg::Twist twist_to_ros(const ::geometry_msgs::msg::v1::Twist &bus) {
  ::geometry_msgs::msg::Twist out;
  out.linear = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.linear());
  out.angular = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.angular());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsTwistMapper
    : public TypedTopicMapper<GeometryMsgsTwistMapper, ::geometry_msgs::msg::Twist> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Twist &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::twist_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Twist bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Twist bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::twist_to_ros(bus);
  }
};
#else
struct GeometryMsgsTwistMapper : TopicMapper {};
#endif

}  // namespace robot_bus
