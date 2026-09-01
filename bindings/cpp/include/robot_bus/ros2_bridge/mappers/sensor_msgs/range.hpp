#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/range.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/range.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::Range range_to_bus(const ::sensor_msgs::msg::Range &msg) {
  ::sensor_msgs::msg::v1::Range bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_radiation_type(msg.radiation_type);
  bus.set_field_of_view(msg.field_of_view);
  bus.set_min_range(msg.min_range);
  bus.set_max_range(msg.max_range);
  bus.set_range(msg.range);
  return bus;
}

inline ::sensor_msgs::msg::Range range_to_ros(const ::sensor_msgs::msg::v1::Range &bus) {
  ::sensor_msgs::msg::Range out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.radiation_type = bus.radiation_type();
  out.field_of_view = bus.field_of_view();
  out.min_range = bus.min_range();
  out.max_range = bus.max_range();
  out.range = bus.range();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsRangeMapper
    : public TypedTopicMapper<SensorMsgsRangeMapper, ::sensor_msgs::msg::Range> {
 public:
  const char *type_name() const override { return "sensor_msgs/msg/Range"; }

  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::Range &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::range_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::Range bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::Range bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::range_to_ros(bus);
  }
};
#else
struct SensorMsgsRangeMapper : TopicMapper {
  const char *type_name() const override { return "sensor_msgs/msg/Range"; }
};
#endif

}  // namespace robot_bus
