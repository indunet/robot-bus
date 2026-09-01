#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/vector3.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/vector3.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Vector3 vector3_to_bus(const ::geometry_msgs::msg::Vector3 &msg) {
  ::geometry_msgs::msg::v1::Vector3 bus;
  bus.set_x(msg.x);
  bus.set_y(msg.y);
  bus.set_z(msg.z);
  return bus;
}

inline ::geometry_msgs::msg::Vector3 vector3_to_ros(const ::geometry_msgs::msg::v1::Vector3 &bus) {
  ::geometry_msgs::msg::Vector3 out;
  out.x = bus.x();
  out.y = bus.y();
  out.z = bus.z();
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsVector3Mapper
    : public TypedTopicMapper<GeometryMsgsVector3Mapper, ::geometry_msgs::msg::Vector3> {
 public:
  const char *type_name() const override { return "geometry_msgs/msg/Vector3"; }

  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Vector3 &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Vector3 bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Vector3 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus);
  }
};
#else
struct GeometryMsgsVector3Mapper : TopicMapper {
  const char *type_name() const override { return "geometry_msgs/msg/Vector3"; }
};
#endif

}  // namespace robot_bus
