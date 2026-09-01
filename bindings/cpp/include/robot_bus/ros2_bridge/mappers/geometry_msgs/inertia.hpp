#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/geometry_msgs/msg/v1/inertia.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <geometry_msgs/msg/inertia.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace geometry_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::geometry_msgs::msg::v1::Inertia inertia_to_bus(const ::geometry_msgs::msg::Inertia &msg) {
  ::geometry_msgs::msg::v1::Inertia bus;
  bus.set_m(msg.m);
  *bus.mutable_com() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.com);
  bus.set_ixx(msg.ixx);
  bus.set_ixy(msg.ixy);
  bus.set_ixz(msg.ixz);
  bus.set_iyy(msg.iyy);
  bus.set_iyz(msg.iyz);
  bus.set_izz(msg.izz);
  return bus;
}

inline ::geometry_msgs::msg::Inertia inertia_to_ros(const ::geometry_msgs::msg::v1::Inertia &bus) {
  ::geometry_msgs::msg::Inertia out;
  out.m = bus.m();
  out.com = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.com());
  out.ixx = bus.ixx();
  out.ixy = bus.ixy();
  out.ixz = bus.ixz();
  out.iyy = bus.iyy();
  out.iyz = bus.iyz();
  out.izz = bus.izz();
  return out;
}
#endif

}  // namespace geometry_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class GeometryMsgsInertiaMapper
    : public TypedTopicMapper<GeometryMsgsInertiaMapper, ::geometry_msgs::msg::Inertia> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::geometry_msgs::msg::Inertia &msg) const {
    auto bus = ros2_bridge_mappers::geometry_msgs::inertia_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::geometry_msgs::msg::Inertia bus_to_ros(BytesView payload) const {
    ::geometry_msgs::msg::v1::Inertia bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::geometry_msgs::inertia_to_ros(bus);
  }
};
#else
struct GeometryMsgsInertiaMapper : TopicMapper {};
#endif

}  // namespace robot_bus
