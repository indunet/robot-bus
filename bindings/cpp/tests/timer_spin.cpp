// Timer fires via background start() (same pattern as pub/sub).
#include "harness.hpp"

#include <atomic>
#include <iostream>

int main() {
  using namespace robot_bus::test;

  auto bus = TestBus::start();
  auto node = bus.make_node("timer_node");

  std::atomic<int> ticks{0};
  auto handle = node.create_timer(0.05, [&] { ticks.fetch_add(1); });
  (void)handle;

  node.start();
  ROBOT_BUS_CHECK(wait_until([&] { return ticks.load() >= 2; }));

  node.shutdown();
  node.wait();
  bus.stop();
  std::cout << "ok: timer_spin\n";
  return 0;
}
