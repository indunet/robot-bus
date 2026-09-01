#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/quaternion.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/quaternion.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Quaternion quaternion_to_bus(const ::geometry_msgs::msg::Quaternion &msg) {
  ::geometry_msgs::msg::v1::Quaternion bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  bus.set_z(msg.z);
  bus.set_w(msg.w);
  return bus;
}

inline ::geometry_msgs::msg::Quaternion quaternion_to_ros(const ::geometry_msgs::msg::v1::Quaternion &bus) {
  ::geometry_msgs::msg::Quaternion out;
  out.x = bus.x();
  out.y = bus.y();
  out.z = bus.z();
  out.w = bus.w();
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsQuaternionMapper
    : public TypedTopicMapper<GeometryMsgsQuaternionMapper, ::geometry_msgs::msg::Quaternion> {
 public:
  const char *type_name() const override { return "geometry_msgs/msg/Quaternion"; }

  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Quaternion &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::quaternion_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Quaternion bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Quaternion bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::quaternion_to_ros(bus);
  }
};
#else
struct GeometryMsgsQuaternionMapper : TopicMapper {
  const char *type_name() const override { return "geometry_msgs/msg/Quaternion"; }
};
#endif

}  // namespace robot_bus
