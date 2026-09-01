#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/point_cloud2.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/sensor_msgs/point_field.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/point_cloud2.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::PointCloud2 point_cloud2_to_bus(const ::sensor_msgs::msg::PointCloud2 &msg) {
  ::sensor_msgs::msg::v1::PointCloud2 bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_height(msg.height);
  bus.set_width(msg.width);
  for (const auto &x : msg.fields) {
    *bus.add_fields() = ::robot_bus::ros2_bridge_mappers::sensor_msgs::point_field_to_bus(x);
  }
  bus.set_is_bigendian(msg.is_bigendian);
  bus.set_point_step(msg.point_step);
  bus.set_row_step(msg.row_step);
  bus.set_data(reinterpret_cast<const char *>(msg.data.data()), msg.data.size());
  bus.set_is_dense(msg.is_dense);
  return bus;
}

inline ::sensor_msgs::msg::PointCloud2 point_cloud2_to_ros(const ::sensor_msgs::msg::v1::PointCloud2 &bus) {
  ::sensor_msgs::msg::PointCloud2 out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.height = bus.height();
  out.width = bus.width();
  out.fields.clear();
  for (const auto &x : bus.fields()) {
    out.fields.push_back(::robot_bus::ros2_bridge_mappers::sensor_msgs::point_field_to_ros(x));
  }
  out.is_bigendian = bus.is_bigendian();
  out.point_step = bus.point_step();
  out.row_step = bus.row_step();
  out.data.assign(bus.data().begin(), bus.data().end());
  out.is_dense = bus.is_dense();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsPointCloud2Mapper
    : public TypedTopicMapper<SensorMsgsPointCloud2Mapper, ::sensor_msgs::msg::PointCloud2> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::PointCloud2 &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::point_cloud2_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::PointCloud2 bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::PointCloud2 bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::point_cloud2_to_ros(bus);
  }
};
#else
struct SensorMsgsPointCloud2Mapper : TopicMapper {};
#endif

}  // namespace robot_bus
