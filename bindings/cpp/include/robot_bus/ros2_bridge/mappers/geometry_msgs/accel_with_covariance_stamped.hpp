#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/accel_covariance.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/accel_with_covariance.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/accel_with_covariance_stamped.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::AccelWithCovarianceStamped accel_with_covariance_stamped_to_bus(const ::geometry_msgs::msg::AccelWithCovarianceStamped &msg) {
  ::geometry_msgs::msg::v1::AccelWithCovarianceStamped bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_accel() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::accel_with_covariance_to_bus(msg.accel);
  return bus;
}

inline ::geometry_msgs::msg::AccelWithCovarianceStamped accel_with_covariance_stamped_to_ros(const ::geometry_msgs::msg::v1::AccelWithCovarianceStamped &bus) {
  ::geometry_msgs::msg::AccelWithCovarianceStamped out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.accel = ::robot_bus::ros2_bridge_mappers::geometry_msgs::accel_with_covariance_to_ros(bus.accel());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsAccelWithCovarianceStampedMapper
    : public TypedTopicMapper<GeometryMsgsAccelWithCovarianceStampedMapper, ::geometry_msgs::msg::AccelWithCovarianceStamped> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::AccelWithCovarianceStamped &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::accel_with_covariance_stamped_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::AccelWithCovarianceStamped bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::AccelWithCovarianceStamped bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::accel_with_covariance_stamped_to_ros(bus);
  }
};
#else
struct GeometryMsgsAccelWithCovarianceStampedMapper : TopicMapper {};
#endif

}  // namespace robot_bus
