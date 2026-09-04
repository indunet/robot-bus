#pragma once

#include <robot_bus/ros2_bridge_mappers.hpp>
#include <robot_bus/ros2_bridge/mappers/convert.hpp>
#include <robot_bus/sensor_msgs/msg/v1/battery_state.pb.h>
#include <robot_bus/ros2_bridge/mappers/std_msgs/header.hpp>

#if defined(ROBOT_BUS_HAS_ROS2)
#include <sensor_msgs/msg/battery_state.hpp>
#include <robot_bus/ros2_bridge_typed.hpp>
#endif

namespace robot_bus {
namespace ros2_bridge_mappers {
namespace sensor_msgs {

#if defined(ROBOT_BUS_HAS_ROS2)
inline ::sensor_msgs::msg::v1::BatteryState battery_state_to_bus(const ::sensor_msgs::msg::BatteryState &msg) {
  ::sensor_msgs::msg::v1::BatteryState bus;
  *bus.mutable_header() = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_bus(msg.header);
  bus.set_voltage(msg.voltage);
  bus.set_current(msg.current);
  bus.set_charge(msg.charge);
  bus.set_capacity(msg.capacity);
  bus.set_design_capacity(msg.design_capacity);
  bus.set_percentage(msg.percentage);
  bus.set_power_supply_status(msg.power_supply_status);
  bus.set_power_supply_health(msg.power_supply_health);
  bus.set_power_supply_technology(msg.power_supply_technology);
  bus.set_present(msg.present);
  for (auto x : msg.cell_voltage) {
    bus.add_cell_voltage(x);
  }
  for (auto x : msg.cell_temperature) {
    bus.add_cell_temperature(x);
  }
  bus.set_location(msg.location.c_str());
  bus.set_serial_number(msg.serial_number.c_str());
  bus.set_temperature(msg.temperature);
  return bus;
}

inline ::sensor_msgs::msg::BatteryState battery_state_to_ros(const ::sensor_msgs::msg::v1::BatteryState &bus) {
  ::sensor_msgs::msg::BatteryState out;
  out.header = ::robot_bus::ros2_bridge_mappers::std_msgs::header_to_ros(bus.header());
  out.voltage = bus.voltage();
  out.current = bus.current();
  out.charge = bus.charge();
  out.capacity = bus.capacity();
  out.design_capacity = bus.design_capacity();
  out.percentage = bus.percentage();
  out.power_supply_status = bus.power_supply_status();
  out.power_supply_health = bus.power_supply_health();
  out.power_supply_technology = bus.power_supply_technology();
  out.present = bus.present();
  out.cell_voltage.assign(bus.cell_voltage().begin(), bus.cell_voltage().end());
  out.cell_temperature.assign(bus.cell_temperature().begin(), bus.cell_temperature().end());
  out.location = bus.location();
  out.serial_number = bus.serial_number();
  out.temperature = bus.temperature();
  return out;
}
#endif

}  // namespace sensor_msgs
}  // namespace ros2_bridge_mappers

#if defined(ROBOT_BUS_HAS_ROS2)
class SensorMsgsBatteryStateMapper
    : public TypedTopicMapper<SensorMsgsBatteryStateMapper, ::sensor_msgs::msg::BatteryState> {
 public:
  std::vector<uint8_t> ros_to_bus(const ::sensor_msgs::msg::BatteryState &msg) const {
    auto bus = ros2_bridge_mappers::sensor_msgs::battery_state_to_bus(msg);
    return encode_pb(bus);
  }

  ::sensor_msgs::msg::BatteryState bus_to_ros(BytesView payload) const {
    ::sensor_msgs::msg::v1::BatteryState bus;
    bus.ParseFromArray(payload.data, static_cast<int>(payload.size));
    return ros2_bridge_mappers::sensor_msgs::battery_state_to_ros(bus);
  }
};
#else
struct SensorMsgsBatteryStateMapper : TopicMapper {};
#endif

}  // namespace robot_bus
