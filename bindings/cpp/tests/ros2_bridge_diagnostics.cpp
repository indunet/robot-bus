#include <robot_bus/ros2_bridge_health.hpp>
#include <cassert>
#include <iostream>
#include <stdexcept>

int main() {
  using namespace robot_bus;
  auto health = std::make_shared<RouteHealth>();
  assert(health->take_idle_event(true, true));
  assert(!health->take_idle_event(true, true));
  health->record_rx();
  assert(!health->is_idle(true, true));
  health->last_rx_ms = RouteHealth::unix_ms() - 16000;
  assert(health->take_idle_event(true, true));
  assert(!health->take_idle_event(true, true));
  health->record_rx();
  health->last_rx_ms = RouteHealth::unix_ms() - 16000;
  assert(health->take_idle_event(true, true));
  assert(!health->is_idle(false, true));
  health->latched = true;
  assert(!health->is_idle(true, true));

  for (auto status : {"succeeded", "timeout", "rejected", "aborted", "cancelled"}) {
    RpcObservation call(health);
    call.finish(status, "test outcome");
  }
  auto stats = health->rpc_snapshot();
  assert(stats.calls == 5 && stats.failures == 3 && stats.timeouts == 1);
  assert(stats.rejected == 1 && stats.cancelled == 1);
  assert(stats.last_status == "cancelled" && stats.last_error == "test outcome");
  assert(health->tx == 1);

  const auto body = bridge_rpc_error_body("aborted", "stopped");
  assert(std::string(body.begin(), body.end()) == std::string("ACTION_ABORTED\0stopped", 22));
  const auto cancelled = bridge_rpc_error_body("cancelled", "motion");
  assert(std::string(cancelled.begin(), cancelled.end()) == std::string("CANCELLED\0motion", 16));
  BridgeRpcError cancelled_err("cancelled", "wait_result: cancelled 'motion'");
  assert(rpc_failure_status(cancelled_err) == "cancelled");
  assert(rpc_failure_status(std::runtime_error("wait_result: cancelled 'motion'")) == "cancelled");
  std::cout << "ROS bridge diagnostics passed\n";
}
