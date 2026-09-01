#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/covariance.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/twist.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/twist_with_covariance.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::TwistWithCovariance twist_with_covariance_to_bus(const ::geometry_msgs::msg::TwistWithCovariance &msg) {
  ::geometry_msgs::msg::v1::TwistWithCovariance bus;
  *bus.mutable_twist() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_bus(msg.twist);
  for (auto x : msg.covariance) {
    bus.add_covariance(x);
  }
  return bus;
}

inline ::geometry_msgs::msg::TwistWithCovariance twist_with_covariance_to_ros(const ::geometry_msgs::msg::v1::TwistWithCovariance &bus) {
  ::geometry_msgs::msg::TwistWithCovariance out;
  out.twist = ::robot_bus::ros2_bridge_mappers::geometry_msgs::twist_to_ros(bus.twist());
  out.covariance.assign(bus.covariance().begin(), bus.covariance().end());
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsTwistWithCovarianceMapper
    : public TypedTopicMapper<GeometryMsgsTwistWithCovarianceMapper, ::geometry_msgs::msg::TwistWithCovariance> {
 public:
  const char *type_name() const override { return "geometry_msgs/msg/TwistWithCovariance"; }

  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::TwistWithCovariance &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::twist_with_covariance_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::TwistWithCovariance bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::TwistWithCovariance bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::twist_with_covariance_to_ros(bus);
  }
};
#else
struct GeometryMsgsTwistWithCovarianceMapper : TopicMapper {
  const char *type_name() const override { return "geometry_msgs/msg/TwistWithCovariance"; }
};
#endif

}  // namespace robot_bus
