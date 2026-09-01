#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/unique_identifier_msgs/msg/v1/uuid.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <unique_identifier_msgs/msg/uuid.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace unique_identifier_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::unique_identifier_msgs::msg::v1::UUID uuid_to_bus(const ::unique_identifier_msgs::msg::UUID &msg) {
  ::unique_identifier_msgs::msg::v1::UUID bus;
  bus.set_uuid(reinterpret_cast<const char *>(msg.uuid.data()), msg.uuid.size());
  return bus;
}

inline ::unique_identifier_msgs::msg::UUID uuid_to_ros(const ::unique_identifier_msgs::msg::v1::UUID &bus) {
  ::unique_identifier_msgs::msg::UUID out;
  out.uuid.assign(bus.uuid().begin(), bus.uuid().end());
  return out;
}
#endif

}  // namespace unique_identifier_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class UniqueIdentifierMsgsUuidMapper
    : public TypedTopicMapper<UniqueIdentifierMsgsUuidMapper, ::unique_identifier_msgs::msg::UUID> {
 public:
  const char *type_name() const override { return "unique_identifier_msgs/msg/UUID"; }

  std::vector<uint8_t> ros_to_bus(const ::unique_identifier_msgs::msg::UUID &msg) const {
    auto bus = ros2_bridge_mappers::unique_identifier_msgs::uuid_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::unique_identifier_msgs::msg::UUID bus_to_ros(BytesView payload) const {
    ::unique_identifier_msgs::msg::v1::UUID bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::unique_identifier_msgs::uuid_to_ros(bus);
  }
};
#else
struct UniqueIdentifierMsgsUuidMapper : TopicMapper {
  const char *type_name() const override { return "unique_identifier_msgs/msg/UUID"; }
};
#endif

}  // namespace robot_bus
