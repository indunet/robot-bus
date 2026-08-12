// Typed IMU pub/sub round-trip against an ephemeral in-process broker.
#include "harness.hpp"

#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

#include <atomic>
#include <iostream>

int main() {
  using namespace robot_bus::test;

  auto bus = TestBus::start();
  auto node = bus.make_node("cpp-pub-sub");
  auto pub = node.create_publisher("/imu");

  std::atomic<bool> got{false};
  double got_z = 0.0;
  auto sub = node.create_subscription("/imu", [&](std::string_view topic, robot_bus::BytesView payload) {
    ROBOT_BUS_CHECK(topic == "/imu");
    sensor_msgs::msg::v1::Imu imu;
    ROBOT_BUS_CHECK(imu.ParseFromArray(payload.data, static_cast<int>(payload.size)));
    got_z = imu.angular_velocity().z();
    got = true;
  });

  node.start();
  sleep_ms(200);  // ZMQ slow joiner

  sensor_msgs::msg::v1::Imu imu;
  imu.mutable_angular_velocity()->set_z(0.1);
  std::string bytes;
  ROBOT_BUS_CHECK(imu.SerializeToString(&bytes));
  pub.publish(bytes);

  ROBOT_BUS_CHECK(wait_until([&] { return got.load(); }));
  ROBOT_BUS_CHECK(got_z == 0.1);

  node.shutdown();
  node.wait();
  bus.stop();
  (void)sub;
  std::cout << "ok: pub_sub_imu\n";
  return 0;
}
