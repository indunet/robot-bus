#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/point_cloud2.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/point_field.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::PointField point_field_to_bus(const ::sensor_msgs::msg::PointField &msg) {
  ::sensor_msgs::msg::v1::PointField bus;
  bus.set_name(msg.name.c_str());
  bus.set_offset(msg.offset);
  bus.set_datatype(static_cast<int32_t>(msg.datatype));
  bus.set_count(msg.count);
  return bus;
}

inline ::sensor_msgs::msg::PointField point_field_to_ros(const ::sensor_msgs::msg::v1::PointField &bus) {
  ::sensor_msgs::msg::PointField out;
  out.name = bus.name();
  out.offset = bus.offset();
  out.datatype = static_cast<uint8_t>(bus.datatype());
  out.count = bus.count();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsPointFieldMapper
    : public TypedTopicMapper<SensorMsgsPointFieldMapper, ::sensor_msgs::msg::PointField> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::PointField &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::point_field_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::PointField bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::PointField bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::point_field_to_ros(bus);
  }
};
#else
struct SensorMsgsPointFieldMapper : TopicMapper {};
#endif

}  // namespace robot_bus
