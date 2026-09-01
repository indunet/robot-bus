#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/foxglove_msgs/msg/v1/geo_json.pb.h>


#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
#include <foxglove_msgs/msg/geo_json.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace foxglove_msgs {

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
inline ::foxglove_msgs::msg::v1::GeoJSON geo_json_to_bus(const ::foxglove_msgs::msg::GeoJSON &msg) {
  ::foxglove_msgs::msg::v1::GeoJSON bus;
  bus.set_geojson(msg.geojson.c_str());
  return bus;
}

inline ::foxglove_msgs::msg::GeoJSON geo_json_to_ros(const ::foxglove_msgs::msg::v1::GeoJSON &bus) {
  ::foxglove_msgs::msg::GeoJSON out;
  out.geojson = bus.geojson();
  return out;
}
#endif

}  // namespace foxglove_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2) && defined(ROBOT_BUS_HAS_FOXGLOVE_MSGS)
class FoxgloveMsgsGeoJsonMapper
    : public TypedTopicMapper<FoxgloveMsgsGeoJsonMapper, ::foxglove_msgs::msg::GeoJSON> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::foxglove_msgs::msg::GeoJSON &msg) const {
    auto bus = ros2_bridge_mappers::foxglove_msgs::geo_json_to_bus(msg);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return std::vector<uint8_t>(bytes.begin(), bytes.end());
  }

  ::foxglove_msgs::msg::GeoJSON bus_to_ros(BytesView payload) const {
    ::foxglove_msgs::msg::v1::GeoJSON bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::foxglove_msgs::geo_json_to_ros(bus);
  }
};
#else
struct FoxgloveMsgsGeoJsonMapper : TopicMapper {};
#endif

}  // namespace robot_bus
