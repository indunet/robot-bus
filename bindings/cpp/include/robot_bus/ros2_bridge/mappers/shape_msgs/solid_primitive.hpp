#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/shape_msgs/msg/v1/solid_primitive.pb.h>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/polygon.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <shape_msgs/msg/solid_primitive.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace shape_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::shape_msgs::msg::v1::SolidPrimitive solid_primitive_to_bus(const ::shape_msgs::msg::SolidPrimitive &msg) {
  ::shape_msgs::msg::v1::SolidPrimitive bus;
  bus.set_type(msg.type);
  for (auto x : msg.dimensions) {
    bus.add_dimensions(x);
  }
  *bus.mutable_polygon() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::polygon_to_bus(msg.polygon);
  return bus;
}

inline ::shape_msgs::msg::SolidPrimitive solid_primitive_to_ros(const ::shape_msgs::msg::v1::SolidPrimitive &bus) {
  ::shape_msgs::msg::SolidPrimitive out;
  out.type = bus.type();
  out.dimensions.assign(bus.dimensions().begin(), bus.dimensions().end());
  out.polygon = ::robot_bus::ros2_bridge_mappers::geometry_msgs::polygon_to_ros(bus.polygon());
  return out;
}
#endif

}  // namespace shape_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class ShapeMsgsSolidPrimitiveMapper
    : public TypedTopicMapper<ShapeMsgsSolidPrimitiveMapper, ::shape_msgs::msg::SolidPrimitive> {
 public:
  const char *type_name() const override { return "shape_msgs/msg/SolidPrimitive"; }

  std::vector<uint8_t> ros_to_bus(const ::shape_msgs::msg::SolidPrimitive &msg) const {
    auto bus = ros2_bridge_mappers::shape_msgs::solid_primitive_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::shape_msgs::msg::SolidPrimitive bus_to_ros(BytesView payload) const {
    ::shape_msgs::msg::v1::SolidPrimitive bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::shape_msgs::solid_primitive_to_ros(bus);
  }
};
#else
struct ShapeMsgsSolidPrimitiveMapper : TopicMapper {
  const char *type_name() const override { return "shape_msgs/msg/SolidPrimitive"; }
};
#endif

}  // namespace robot_bus
