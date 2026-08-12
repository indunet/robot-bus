// Typed MessageLite wrappers (requires ROBOT_BUS_BUILD_MSGS).
#include "harness.hpp"

#include <robot_bus/typed.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

#include <atomic>
#include <iostream>

int main() {
  using namespace robot_bus;
  using namespace robot_bus::test;
  using sensor_msgs::msg::v1::Imu;

  auto bus = TestBus::start();
  auto node = bus.make_node("cpp-typed");

  std::atomic<bool> got{false};
  double got_z = 0.0;
  auto sub = create_subscription<Imu>(
      node, "/imu", [&](std::string_view topic, const Imu &msg) {
        ROBOT_BUS_CHECK(topic == "/imu");
        got_z = msg.angular_velocity().z();
        got = true;
      });

  auto pub = create_publisher<Imu>(node, "/imu");
  node.start();
  sleep_ms(200);

  Imu imu;
  imu.mutable_angular_velocity()->set_z(0.25);
  pub.publish(imu);

  ROBOT_BUS_CHECK(wait_until([&] { return got.load(); }));
  ROBOT_BUS_CHECK(got_z == 0.25);

  node.shutdown();
  node.wait();
  bus.stop();
  (void)sub;
  std::cout << "ok: typed_api\n";
  return 0;
}
