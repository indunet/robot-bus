#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/nav_sat_status.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/nav_sat_status.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::NavSatStatus nav_sat_status_to_bus(const ::sensor_msgs::msg::NavSatStatus &msg) {
  ::sensor_msgs::msg::v1::NavSatStatus bus;
  bus.set_status(static_cast<int32_t>(msg.status));
  bus.set_service(static_cast<int32_t>(msg.service));
  return bus;
}

inline ::sensor_msgs::msg::NavSatStatus nav_sat_status_to_ros(const ::sensor_msgs::msg::v1::NavSatStatus &bus) {
  ::sensor_msgs::msg::NavSatStatus out;
  out.status = static_cast<int8_t>(bus.status());
  out.service = static_cast<uint16_t>(bus.service());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsNavSatStatusMapper
    : public TypedTopicMapper<SensorMsgsNavSatStatusMapper, ::sensor_msgs::msg::NavSatStatus> {
 public:
  const char *type_name() const override { return "sensor_msgs/msg/NavSatStatus"; }

  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::NavSatStatus &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::nav_sat_status_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::NavSatStatus bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::NavSatStatus bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::nav_sat_status_to_ros(bus);
  }
};
#else
struct SensorMsgsNavSatStatusMapper : TopicMapper {
  const char *type_name() const override { return "sensor_msgs/msg/NavSatStatus"; }
};
#endif

}  // namespace robot_bus
