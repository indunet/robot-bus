#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/pose2d.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/pose2_d.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Pose2D pose2_d_to_bus(const ::geometry_msgs::msg::Pose2D &msg) {
  ::geometry_msgs::msg::v1::Pose2D bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  bus.set_theta(msg.theta);
  return bus;
}

inline ::geometry_msgs::msg::Pose2D pose2_d_to_ros(const ::geometry_msgs::msg::v1::Pose2D &bus) {
  ::geometry_msgs::msg::Pose2D out;
  out.x = bus.x();
  out.y = bus.y();
  out.theta = bus.theta();
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPose2DMapper
    : public TypedTopicMapper<GeometryMsgsPose2DMapper, ::geometry_msgs::msg::Pose2D> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Pose2D &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::pose2_d_to_bus(msg);
    return encode_pb(bus);
  }

  ::geometry_msgs::msg::Pose2D bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Pose2D bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::pose2_d_to_ros(bus);
  }
};
#else
struct GeometryMsgsPose2DMapper : TopicMapper {};
#endif

}  // namespace robot_bus
