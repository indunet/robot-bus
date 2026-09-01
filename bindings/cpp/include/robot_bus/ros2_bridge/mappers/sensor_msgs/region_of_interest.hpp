#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/camera_info.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/region_of_interest.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::RegionOfInterest region_of_interest_to_bus(const ::sensor_msgs::msg::RegionOfInterest &msg) {
  ::sensor_msgs::msg::v1::RegionOfInterest bus;
  bus.set_x_offset(msg.x_offset);
  bus.set_y_offset(msg.y_offset);
  bus.set_height(msg.height);
  bus.set_width(msg.width);
  bus.set_do_rectify(msg.do_rectify);
  return bus;
}

inline ::sensor_msgs::msg::RegionOfInterest region_of_interest_to_ros(const ::sensor_msgs::msg::v1::RegionOfInterest &bus) {
  ::sensor_msgs::msg::RegionOfInterest out;
  out.x_offset = bus.x_offset();
  out.y_offset = bus.y_offset();
  out.height = bus.height();
  out.width = bus.width();
  out.do_rectify = bus.do_rectify();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsRegionOfInterestMapper
    : public TypedTopicMapper<SensorMsgsRegionOfInterestMapper, ::sensor_msgs::msg::RegionOfInterest> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::RegionOfInterest &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::region_of_interest_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::sensor_msgs::msg::RegionOfInterest bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::RegionOfInterest bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::region_of_interest_to_ros(bus);
  }
};
#else
struct SensorMsgsRegionOfInterestMapper : TopicMapper {};
#endif

}  // namespace robot_bus
