// Cross-language interop peer (env-driven endpoints).
// Roles: svc-server | act-client
#include "harness.hpp"

#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>
#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>

#include <chrono>
#include <cstdlib>
#include <iostream>
#include <string>
#include <thread>
#include <utility>
#include <vector>

namespace {

constexpr const char *kService = "/interop/set_bool";
constexpr const char *kAction = "/interop/fibonacci";

std::string require_env(const char *key) {
  const char *v = std::getenv(key);
  if (!v || !*v) {
    std::cerr << "missing env " << key << "\n";
    std::exit(1);
  }
  return std::string(v);
}

struct EndpointBag {
  std::string message_xsub;
  std::string message_xpub;
  std::string service_frontend;
  std::string service_backend;
  std::string action_frontend;
  std::string action_backend;

  static EndpointBag from_env() {
    EndpointBag e;
    e.message_xsub = require_env("ROBOT_BUS_MESSAGE_XSUB");
    e.message_xpub = require_env("ROBOT_BUS_MESSAGE_XPUB");
    e.service_frontend = require_env("ROBOT_BUS_SERVICE_FRONTEND");
    e.service_backend = require_env("ROBOT_BUS_SERVICE_BACKEND");
    e.action_frontend = require_env("ROBOT_BUS_ACTION_FRONTEND");
    e.action_backend = require_env("ROBOT_BUS_ACTION_BACKEND");
    return e;
  }

  RobotBusNodeOptions to_opts() const {
    RobotBusNodeOptions opts{};
    opts.host = nullptr;
    opts.transport = nullptr;
    opts.ws_url = nullptr;
    opts.message_xsub = message_xsub.c_str();
    opts.message_xpub = message_xpub.c_str();
    opts.service_frontend = service_frontend.c_str();
    opts.service_backend = service_backend.c_str();
    opts.action_frontend = action_frontend.c_str();
    opts.action_backend = action_backend.c_str();
    return opts;
  }
};

void run_svc_server() {
  auto ends = EndpointBag::from_env();
  auto opts = ends.to_opts();
  robot_bus::Node node("interop_cpp_svc", opts);
  auto svc = node.create_service(kService, [](robot_bus::BytesView body) {
    std_srvs::srv::v1::SetBoolRequest req;
    ROBOT_BUS_CHECK(req.ParseFromArray(body.data, static_cast<int>(body.size)));
    std_srvs::srv::v1::SetBoolResponse resp;
    resp.set_success(true);
    resp.set_message(std::string("set:") + (req.data() ? "true" : "false"));
    std::string out;
    ROBOT_BUS_CHECK(resp.SerializeToString(&out));
    return std::vector<uint8_t>(out.begin(), out.end());
  });
  node.start();
  std::cout << "READY" << std::endl;
  std::this_thread::sleep_for(std::chrono::seconds(15));
  node.shutdown();
  node.wait();
  (void)svc;
}

void run_act_client() {
  std::this_thread::sleep_for(std::chrono::milliseconds(400));
  auto ends = EndpointBag::from_env();
  auto opts = ends.to_opts();
  robot_bus::Node node("interop_cpp_act_client", opts);
  auto client = node.create_action_client(kAction);

  example_interfaces::action::v1::FibonacciGoal goal;
  goal.set_order(5);
  std::string goal_bytes;
  ROBOT_BUS_CHECK(goal.SerializeToString(&goal_bytes));

  auto handle = client.send_goal(goal_bytes, {}, nullptr, 10.0);
  auto event = handle.wait_result(10.0);
  ROBOT_BUS_CHECK(event.kind == "RESULT");
  example_interfaces::action::v1::FibonacciResult result;
  ROBOT_BUS_CHECK(
      result.ParseFromArray(event.body.data(), static_cast<int>(event.body.size())));
  ROBOT_BUS_CHECK(result.sequence_size() == 5);
  ROBOT_BUS_CHECK(result.sequence(0) == 0);
  ROBOT_BUS_CHECK(result.sequence(1) == 1);
  ROBOT_BUS_CHECK(result.sequence(2) == 1);
  ROBOT_BUS_CHECK(result.sequence(3) == 2);
  ROBOT_BUS_CHECK(result.sequence(4) == 3);
  std::cout << "READY" << std::endl;
}

}  // namespace

int main() {
  const char *role = std::getenv("ROBOT_BUS_INTEROP_ROLE");
  if (!role) {
    std::cerr << "ROBOT_BUS_INTEROP_ROLE required\n";
    return 1;
  }
  const std::string r(role);
  if (r == "svc-server") {
    run_svc_server();
  } else if (r == "act-client") {
    run_act_client();
  } else {
    std::cerr << "unknown role: " << r << "\n";
    return 1;
  }
  return 0;
}
