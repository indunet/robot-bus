// Custom Ros2Bridge: ROS .srv + our bus .proto + TypedServiceMapper.
//
// Two interface definitions (fields must match):
//   examples/ros2_bridge/ros2/my_pkg/srv/AddTwoInts.srv
//   examples/ros2_bridge/proto/my_pkg/srv/v1/add_two_ints.proto
//
// Generated bus stubs: examples/ros2_bridge/generated/my_pkg/...
//
// Runtime smoke uses system example_interfaces/srv/AddTwoInts (identical
// fields) so you need not colcon-build my_pkg first.
//
//   just examples-cpp-ros2
//   ./examples/build/ros2_bridge_custom_add_two_ints
//   ros2 service call /examples/add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 40}"
#include <robot_bus/node.hpp>
#include <robot_bus/ros2_bridge.hpp>
#include <robot_bus/typed.hpp>

#include <example_interfaces/srv/add_two_ints.hpp>
#include "my_pkg/srv/v1/add_two_ints.pb.h"

#include <chrono>
#include <iostream>
#include <memory>
#include <thread>

struct AddTwoIntsServiceMapper
    : robot_bus::TypedServiceMapper<AddTwoIntsServiceMapper,
                                    example_interfaces::srv::AddTwoInts> {
  // Smoke: system type with the same fields as ros2/my_pkg/srv/AddTwoInts.srv.
  const char *type_name() const override {
    return "example_interfaces/srv/AddTwoInts";
  }

  std::vector<uint8_t> ros_req_to_bus(const Request &req) const {
    my_pkg::srv::v1::AddTwoIntsRequest bus;
    bus.set_a(req.a);
    bus.set_b(req.b);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return {bytes.begin(), bytes.end()};
  }

  Request bus_req_to_ros(robot_bus::BytesView body) const {
    my_pkg::srv::v1::AddTwoIntsRequest bus;
    bus.ParseFromArray(body.data, static_cast<int>(body.size));
    Request out;
    out.a = bus.a();
    out.b = bus.b();
    return out;
  }

  std::vector<uint8_t> ros_resp_to_bus(const Response &resp) const {
    my_pkg::srv::v1::AddTwoIntsResponse bus;
    bus.set_sum(resp.sum);
    std::string bytes;
    bus.SerializeToString(&bytes);
    return {bytes.begin(), bytes.end()};
  }

  Response bus_resp_to_ros(robot_bus::BytesView body) const {
    my_pkg::srv::v1::AddTwoIntsResponse bus;
    bus.ParseFromArray(body.data, static_cast<int>(body.size));
    Response out;
    out.sum = bus.sum();
    return out;
  }
};

int main() {
  if (!robot_bus::ros2_available()) {
    std::cerr << "ROS 2 bridge not linked (need robot_bus_ros2_bridge / "
                 "just examples-cpp-ros2)\n";
    return 1;
  }

  robot_bus::Node bus("examples_add_two_ints_bus");
  auto svc = robot_bus::create_service<my_pkg::srv::v1::AddTwoIntsRequest,
                                       my_pkg::srv::v1::AddTwoIntsResponse>(
      bus, "/examples/add_two_ints",
      [](const my_pkg::srv::v1::AddTwoIntsRequest &req) {
        my_pkg::srv::v1::AddTwoIntsResponse resp;
        resp.set_sum(req.a() + req.b());
        return resp;
      });
  (void)svc;
  bus.start();

  auto bridge =
      robot_bus::Ros2Bridge::New("examples_ros2_bridge_custom")
          .bus_tcp("localhost")
          .service("/examples/add_two_ints", "/examples/add_two_ints")
          .mapper(std::make_shared<AddTwoIntsServiceMapper>())
          .direction(robot_bus::Direction::Ros2ToBus)
          .timeout(5.0)
          .add()
          .build();

  std::cout << "custom my_pkg AddTwoInts bridge on /examples/add_two_ints "
               "(Ros2ToBus + in-process bus server; Ctrl+C to stop)\n";
  while (true) {
    bus.spin_once(0.01);
    bridge.spin_once(0.01);
    std::this_thread::sleep_for(std::chrono::milliseconds(1));
  }
  return 0;
}
