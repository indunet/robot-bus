// Publish a few Imu messages on /examples/imu.
#include <robot_bus/node.hpp>
#include <robot_bus/typed.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

#include <chrono>
#include <iostream>
#include <thread>

int main() {
  robot_bus::Node node("examples_imu_talker");
  auto pub = robot_bus::create_publisher<sensor_msgs::msg::v1::Imu>(node, "/examples/imu");
  node.start();
  std::this_thread::sleep_for(std::chrono::milliseconds(300));

  for (int i = 0; i < 5; ++i) {
    sensor_msgs::msg::v1::Imu imu;
    imu.mutable_linear_acceleration()->set_z(9.8 + i * 0.01);
    pub.publish(imu);
    std::cout << "published Imu #" << i << "\n";
    std::this_thread::sleep_for(std::chrono::milliseconds(200));
  }
  node.shutdown();
  return 0;
}
