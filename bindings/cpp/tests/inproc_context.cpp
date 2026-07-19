#include "harness.hpp"

#include <atomic>
#include <chrono>
#include <string>
#include <thread>

int main() {
  using namespace robot_bus;
  using namespace robot_bus::test;

  Context ctx;
  RobotBusBrokerOptions opts{};
  opts.no_console = 1;
  // Keep default binds (incl. inproc); do not force tcp_only.
  Broker broker(ctx, opts);
  std::this_thread::sleep_for(std::chrono::milliseconds(150));

  std::atomic<int> hits{0};
  auto sub = Node::inproc_with_context(ctx, "inproc-sub");
  sub.create_subscription("/inproc/demo", [&](std::string_view, BytesView payload) {
    ROBOT_BUS_CHECK(std::string(reinterpret_cast<const char *>(payload.data), payload.size) ==
                    "hello-inproc");
    hits.fetch_add(1);
  });
  sub.start();
  std::this_thread::sleep_for(std::chrono::milliseconds(100));

  auto pub = Node::inproc_with_context(ctx, "inproc-pub");
  auto topic = pub.create_publisher("/inproc/demo");
  const char *msg = "hello-inproc";
  topic.publish(BytesView(reinterpret_cast<const uint8_t *>(msg), 12));

  auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(3);
  while (hits.load() < 1 && std::chrono::steady_clock::now() < deadline) {
    std::this_thread::sleep_for(std::chrono::milliseconds(20));
  }
  ROBOT_BUS_CHECK(hits.load() >= 1);

  sub.stop();
  broker.stop();
  return 0;
}
