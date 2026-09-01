#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/quaternion.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/vector3.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/imu.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::Imu imu_to_bus(const ::sensor_msgs::msg::Imu &msg) {
  ::sensor_msgs::msg::v1::Imu bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  *bus.mutable_orientation() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_bus(msg.orientation);
  for (auto x : msg.orientation_covariance) {
    bus.add_orientation_covariance(x);
  }
  *bus.mutable_angular_velocity() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.angular_velocity);
  for (auto x : msg.angular_velocity_covariance) {
    bus.add_angular_velocity_covariance(x);
  }
  *bus.mutable_linear_acceleration() = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_bus(msg.linear_acceleration);
  for (auto x : msg.linear_acceleration_covariance) {
    bus.add_linear_acceleration_covariance(x);
  }
  return bus;
}

inline ::sensor_msgs::msg::Imu imu_to_ros(const ::sensor_msgs::msg::v1::Imu &bus) {
  ::sensor_msgs::msg::Imu out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.orientation = ::robot_bus::ros2_bridge_mappers::geometry_msgs::quaternion_to_ros(bus.orientation());
  out.orientation_covariance.assign(bus.orientation_covariance().begin(), bus.orientation_covariance().end());
  out.angular_velocity = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.angular_velocity());
  out.angular_velocity_covariance.assign(bus.angular_velocity_covariance().begin(), bus.angular_velocity_covariance().end());
  out.linear_acceleration = ::robot_bus::ros2_bridge_mappers::geometry_msgs::vector3_to_ros(bus.linear_acceleration());
  out.linear_acceleration_covariance.assign(bus.linear_acceleration_covariance().begin(), bus.linear_acceleration_covariance().end());
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsImuMapper
    : public TypedTopicMapper<SensorMsgsImuMapper, ::sensor_msgs::msg::Imu> {
 public:
  const char *type_name() const override { return "sensor_msgs/msg/Imu"; }

  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::Imu &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::imu_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::Imu bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::Imu bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::imu_to_ros(bus);
  }
};
#else
struct SensorMsgsImuMapper : TopicMapper {
  const char *type_name() const override { return "sensor_msgs/msg/Imu"; }
};
#endif

}  // namespace robot_bus
