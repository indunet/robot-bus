// Subscribe to /examples/imu (typed Imu). Requires a running robot-bus-broker.
#include <robot_bus/node.hpp>
#include <robot_bus/typed.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

#include <iostream>

int main() {
  robot_bus::Node node("examples_imu_listener");
  auto sub = robot_bus::create_subscription<sensor_msgs::msg::v1::Imu>(
      node, "/examples/imu",
      [](std::string_view topic, const sensor_msgs::msg::v1::Imu &imu) {
        double z = imu.linear_acceleration().z();
        std::cout << topic << ": linear_acceleration.z=" << z << "\n";
      });
  (void)sub;
  std::cout << "listening on /examples/imu (Ctrl+C to stop)\n";
  node.start();
  node.spin();
  return 0;
}
