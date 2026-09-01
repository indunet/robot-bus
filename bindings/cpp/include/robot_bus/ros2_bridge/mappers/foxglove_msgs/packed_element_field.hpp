#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/packed_element_field.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/packed_element_field.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::PackedElementField packed_element_field_to_bus(const ::foxglove_msgs::msg::PackedElementField &msg) {
  ::foxglove_msgs::msg::v1::PackedElementField bus;
  bus.set_name(msg.name.c_str());
  bus.set_offset(msg.offset);
  bus.set_type(static_cast<int32_t>(msg.type));
  return bus;
}

inline ::foxglove_msgs::msg::PackedElementField packed_element_field_to_ros(const ::foxglove_msgs::msg::v1::PackedElementField &bus) {
  ::foxglove_msgs::msg::PackedElementField out;
  out.name = bus.name();
  out.offset = bus.offset();
  out.type = bus.type();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsPackedElementFieldMapper
    : public TypedTopicMapper<FoxgloveMsgsPackedElementFieldMapper, ::foxglove_msgs::msg::PackedElementField> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::PackedElementField &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::packed_element_field_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::PackedElementField bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::PackedElementField bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::packed_element_field_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsPackedElementFieldMapper : TopicMapper {};
#endif

}  // namespace robot_bus
