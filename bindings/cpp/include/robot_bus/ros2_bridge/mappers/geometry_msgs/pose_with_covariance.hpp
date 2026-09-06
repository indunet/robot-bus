#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/covariance.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/pose_with_covariance.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::PoseWithCovariance pose_with_covariance_to_bus(const ::geometry_msgs::msg::PoseWithCovariance &msg) {
  ::geometry_msgs::msg::v1::PoseWithCovariance bus;
  *bus.mutable_pose() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_bus(msg.pose);
  for (auto x : msg.covariance) {
    bus.add_covariance(x);
  }
  return bus;
}

inline ::geometry_msgs::msg::PoseWithCovariance pose_with_covariance_to_ros(const ::geometry_msgs::msg::v1::PoseWithCovariance &bus) {
  ::geometry_msgs::msg::PoseWithCovariance out;
  out.pose = ::robot_bus::ros2_bridge_mappers::geometry_msgs::pose_to_ros(bus.pose());
  ::robot_bus::ros2_bridge_mappers::copy_seq(out.covariance, bus.covariance());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsPoseWithCovarianceMapper
    : public TypedTopicMapper<GeometryMsgsPoseWithCovarianceMapper, ::geometry_msgs::msg::PoseWithCovariance> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::PoseWithCovariance &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::pose_with_covariance_to_bus(msg);
    return encode_pb(bus);
  }

  ::geometry_msgs::msg::PoseWithCovariance bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::PoseWithCovariance bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::pose_with_covariance_to_ros(bus);
  }
};
#else
struct GeometryMsgsPoseWithCovarianceMapper : TopicMapper {};
#endif

}  // namespace robot_bus
