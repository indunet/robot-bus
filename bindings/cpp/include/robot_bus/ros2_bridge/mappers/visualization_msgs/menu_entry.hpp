#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/visualization_msgs/msg/v1/menu_entry.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <visualization_msgs/msg/menu_entry.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace visualization_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::visualization_msgs::msg::v1::MenuEntry menu_entry_to_bus(const ::visualization_msgs::msg::MenuEntry &msg) {
  ::visualization_msgs::msg::v1::MenuEntry bus;
  bus.set_id(msg.id);
  bus.set_parent_id(msg.parent_id);
  bus.set_title(msg.title.c_str());
  bus.set_command(msg.command.c_str());
  bus.set_command_type(msg.command_type);
  return bus;
}

inline ::visualization_msgs::msg::MenuEntry menu_entry_to_ros(const ::visualization_msgs::msg::v1::MenuEntry &bus) {
  ::visualization_msgs::msg::MenuEntry out;
  out.id = bus.id();
  out.parent_id = bus.parent_id();
  out.title = bus.title();
  out.command = bus.command();
  out.command_type = bus.command_type();
  return out;
}
#endif

}  // namespace visualization_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class VisualizationMsgsMenuEntryMapper
    : public TypedTopicMapper<VisualizationMsgsMenuEntryMapper, ::visualization_msgs::msg::MenuEntry> {
 public:
  const char *type_name() const override { return "visualization_msgs/msg/MenuEntry"; }

  std::vector<uint8_t> ros_to_bus(const ::visualization_msgs::msg::MenuEntry &msg) const {
    auto bus = ros2_bridge_mappers::visualization_msgs::menu_entry_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::visualization_msgs::msg::MenuEntry bus_to_ros(BytesView payload) const {
    ::visualization_msgs::msg::v1::MenuEntry bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::visualization_msgs::menu_entry_to_ros(bus);
  }
};
#else
struct VisualizationMsgsMenuEntryMapper : TopicMapper {
  const char *type_name() const override { return "visualization_msgs/msg/MenuEntry"; }
};
#endif

}  // namespace robot_bus
