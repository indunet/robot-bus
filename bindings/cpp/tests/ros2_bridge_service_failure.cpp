// ROS caller -> bus service that does not exist, against real rclcpp:
//  - a mapper with a natural failure shape (Trigger) must reply `success = false`;
//  - a mapper without `error_response` must send NO reply (the caller times out),
//    never a default-constructed Response that looks like a real result.
#include <robot_bus/ros2_bridge.hpp>

#include <example_interfaces/srv/add_two_ints.hpp>
#include <rclcpp/rclcpp.hpp>
#include <std_srvs/srv/trigger.hpp>

#include "harness.hpp"

#include <chrono>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

namespace {

using AddTwoInts = example_interfaces::srv::AddTwoInts;

/// Response has no failure field and the mapper has no `error_response`.
struct AddNoErrorResponse : robot_bus::TypedServiceMapper<AddNoErrorResponse, AddTwoInts> {
  const char *type_name() const override { return "example_interfaces/srv/AddTwoInts"; }

  std::vector<uint8_t> ros_req_to_bus(const Request &req) const {
    std::vector<uint8_t> out(16);
    std::memcpy(out.data(), &req.a, 8);
    std::memcpy(out.data() + 8, &req.b, 8);
    return out;
  }
  Request bus_req_to_ros(robot_bus::BytesView) const { return Request{}; }
  std::vector<uint8_t> ros_resp_to_bus(const Response &resp) const {
    std::vector<uint8_t> out(8);
    std::memcpy(out.data(), &resp.sum, 8);
    return out;
  }
  Response bus_resp_to_ros(robot_bus::BytesView body) const {
    Response out;
    if (body.size >= 8) std::memcpy(&out.sum, body.data, 8);
    return out;
  }
};

/// Pump the bridge and the ROS client until `done()` or `timeout` elapses.
template <typename Done>
bool pump(robot_bus::Ros2Bridge &bridge, const rclcpp::Node::SharedPtr &client_node, Done done,
          std::chrono::milliseconds timeout) {
  const auto deadline = std::chrono::steady_clock::now() + timeout;
  while (std::chrono::steady_clock::now() < deadline) {
    if (done()) return true;
    bridge.spin_once(0.005);
    rclcpp::spin_some(client_node);
  }
  return done();
}

}  // namespace

int main() {
  using namespace robot_bus::test;
  using namespace std::chrono_literals;

  rclcpp::init(0, nullptr);
  auto bus = TestBus::start();

  const auto ros_qos = robot_bus::TopicQos::ros_default();
  const auto bus_qos = robot_bus::TopicQos::bus();
  const std::string pid = std::to_string(::getpid());
  const std::string trig_name = "/cpp_fail_trigger_" + pid;
  const std::string add_name = "/cpp_fail_add_" + pid;

  auto bridge = robot_bus::Ros2Bridge::New("cpp_service_failure")
                    .bus_discover("http://" + bus.api_listen, 10.0)
                    .service()
                    .from_ros(trig_name, ros_qos)
                    .to_bus("trigger_fail_" + pid, bus_qos)
                    .mapper(robot_bus::TriggerServiceMapper{})
                    .timeout(1.0)
                    .add()
                    .service()
                    .from_ros(add_name, ros_qos)
                    .to_bus("add_fail_" + pid, bus_qos)
                    .mapper(std::make_shared<AddNoErrorResponse>())
                    .timeout(1.0)
                    .add()
                    .build();

  auto client_node = std::make_shared<rclcpp::Node>("cpp_service_failure_client");
  auto trig = client_node->create_client<std_srvs::srv::Trigger>(trig_name);
  auto add = client_node->create_client<AddTwoInts>(add_name);
  ROBOT_BUS_CHECK(pump(bridge, client_node, [&] { return trig->service_is_ready() && add->service_is_ready(); },
                       5000ms));

  // 1) Trigger: failure is reported in-band.
  auto trig_future = trig->async_send_request(std::make_shared<std_srvs::srv::Trigger::Request>());
  ROBOT_BUS_CHECK(pump(bridge, client_node,
                       [&] { return trig_future.wait_for(0s) == std::future_status::ready; }, 6000ms));
  auto trig_resp = trig_future.get();
  ROBOT_BUS_CHECK(!trig_resp->success);
  ROBOT_BUS_CHECK(trig_resp->message.find("bus call failed") != std::string::npos);
  std::cout << "Trigger failure reply: success=false message=\"" << trig_resp->message << "\"\n";

  // 2) AddTwoInts without error_response: no reply at all. The bridge call times out
  //    after 1s; watch well past that for a (wrong) reply.
  auto req = std::make_shared<AddTwoInts::Request>();
  req->a = 2;
  req->b = 40;
  auto add_future = add->async_send_request(req);
  const bool replied = pump(bridge, client_node,
                            [&] { return add_future.wait_for(0s) == std::future_status::ready; }, 4000ms);
  if (replied) {
    std::cerr << "bridge failure reached the ROS caller as a response: sum=" << add_future.get()->sum
              << "\n";
    return 1;
  }
  std::cout << "AddTwoInts failure: no reply sent (caller would time out)\n";

  bus.stop();
  rclcpp::shutdown();
  std::cout << "ROS bridge service failure passed\n";
  return 0;
}
