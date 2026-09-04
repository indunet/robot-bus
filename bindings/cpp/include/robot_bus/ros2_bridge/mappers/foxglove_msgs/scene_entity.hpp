#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/scene_entity.pb.h>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/key_value_pair.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/arrow_primitive.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/cube_primitive.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/sphere_primitive.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/cylinder_primitive.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/line_primitive.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/triangle_list_primitive.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/text_primitive.hpp>
#include <robot_bus/ros2_bridge/mappers/foxglove_msgs/model_primitive.hpp>

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/scene_entity.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::SceneEntity scene_entity_to_bus(const ::foxglove_msgs::msg::SceneEntity &msg) {
  ::foxglove_msgs::msg::v1::SceneEntity bus;
  *bus.mutable_timestamp() = ::robot_bus::ros2_bridge_mappers::time_to_timestamp(msg.timestamp);
  bus.set_frame_id(msg.frame_id.c_str());
  bus.set_id(msg.id.c_str());
  *bus.mutable_lifetime() = ::robot_bus::ros2_bridge_mappers::duration_to_proto(msg.lifetime);
  bus.set_frame_locked(msg.frame_locked);
  for (const auto &x : msg.metadata) {
    *bus.add_metadata() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_bus(x);
  }
  for (const auto &x : msg.arrows) {
    *bus.add_arrows() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::arrow_primitive_to_bus(x);
  }
  for (const auto &x : msg.cubes) {
    *bus.add_cubes() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::cube_primitive_to_bus(x);
  }
  for (const auto &x : msg.spheres) {
    *bus.add_spheres() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::sphere_primitive_to_bus(x);
  }
  for (const auto &x : msg.cylinders) {
    *bus.add_cylinders() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::cylinder_primitive_to_bus(x);
  }
  for (const auto &x : msg.lines) {
    *bus.add_lines() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::line_primitive_to_bus(x);
  }
  for (const auto &x : msg.triangles) {
    *bus.add_triangles() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::triangle_list_primitive_to_bus(x);
  }
  for (const auto &x : msg.texts) {
    *bus.add_texts() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::text_primitive_to_bus(x);
  }
  for (const auto &x : msg.models) {
    *bus.add_models() = ::robot_bus::ros2_bridge_mappers::foxglove_msgs::model_primitive_to_bus(x);
  }
  return bus;
}

inline ::foxglove_msgs::msg::SceneEntity scene_entity_to_ros(const ::foxglove_msgs::msg::v1::SceneEntity &bus) {
  ::foxglove_msgs::msg::SceneEntity out;
  out.timestamp = ::robot_bus::ros2_bridge_mappers::timestamp_to_time(bus.timestamp());
  out.frame_id = bus.frame_id();
  out.id = bus.id();
  out.lifetime = ::robot_bus::ros2_bridge_mappers::proto_to_duration(bus.lifetime());
  out.frame_locked = bus.frame_locked();
  out.metadata.clear();
  for (const auto &x : bus.metadata()) {
    out.metadata.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::key_value_pair_to_ros(x));
  }
  out.arrows.clear();
  for (const auto &x : bus.arrows()) {
    out.arrows.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::arrow_primitive_to_ros(x));
  }
  out.cubes.clear();
  for (const auto &x : bus.cubes()) {
    out.cubes.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::cube_primitive_to_ros(x));
  }
  out.spheres.clear();
  for (const auto &x : bus.spheres()) {
    out.spheres.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::sphere_primitive_to_ros(x));
  }
  out.cylinders.clear();
  for (const auto &x : bus.cylinders()) {
    out.cylinders.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::cylinder_primitive_to_ros(x));
  }
  out.lines.clear();
  for (const auto &x : bus.lines()) {
    out.lines.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::line_primitive_to_ros(x));
  }
  out.triangles.clear();
  for (const auto &x : bus.triangles()) {
    out.triangles.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::triangle_list_primitive_to_ros(x));
  }
  out.texts.clear();
  for (const auto &x : bus.texts()) {
    out.texts.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::text_primitive_to_ros(x));
  }
  out.models.clear();
  for (const auto &x : bus.models()) {
    out.models.push_back(::robot_bus::ros2_bridge_mappers::foxglove_msgs::model_primitive_to_ros(x));
  }
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsSceneEntityMapper
    : public TypedTopicMapper<FoxgloveMsgsSceneEntityMapper, ::foxglove_msgs::msg::SceneEntity> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::SceneEntity &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::scene_entity_to_bus(msg);
    return encode_pb(bus);
  }

  ::foxglove_msgs::msg::SceneEntity bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::SceneEntity bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::scene_entity_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsSceneEntityMapper : TopicMapper {};
#endif

}  // namespace robot_bus
