#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/mesh_file.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/mesh_file.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::MeshFile mesh_file_to_bus(const ::visualization_msgs::msg::MeshFile &msg) {
  ::visualization_msgs::msg::v1::MeshFile bus;
  bus.set_filename(msg.filename.c_str());
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  return bus;
}

inline ::visualization_msgs::msg::MeshFile mesh_file_to_ros(const ::visualization_msgs::msg::v1::MeshFile &bus) {
  ::visualization_msgs::msg::MeshFile out;
  out.filename = bus.filename();
  out.data.assign(bus.data().begin(), bus.data().end());
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsMeshFileMapper
    : public TypedTopicMapper<VisualizationMsgsMeshFileMapper, ::visualization_msgs::msg::MeshFile> {
 public:
  const char *type_name() const override { return "visualization_msgs/msg/MeshFile"; }

  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::MeshFile &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::mesh_file_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::visualization_msgs::msg::MeshFile bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::MeshFile bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::mesh_file_to_ros(bus);
  }
};
#else
struct VisualizationMsgsMeshFileMapper : TopicMapper {
  const char *type_name() const override { return "visualization_msgs/msg/MeshFile"; }
};
#endif

}  // namespace robot_bus
