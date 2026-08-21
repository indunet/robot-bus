#include "harness.hpp"

#include <atomic>
#include <chrono>
#include <string>
#include <thread>
#include <utility>
#include <vector>

namespace {

std::string bytes_to_string(const std::vector<uint8_t> &body) {
  return std::string(reinterpret_cast<const char *>(body.data()), body.size());
}

std::string bytes_to_string(robot_bus::BytesView payload) {
  return std::string(reinterpret_cast<const char *>(payload.data), payload.size);
}

}  // namespace

int main() {
  using namespace robot_bus;
  using namespace robot_bus::test;

  Context ctx;
  const std::string message_xsub = "tcp://127.0.0.1:" + std::to_string(free_port());
  const std::string message_xpub = "tcp://127.0.0.1:" + std::to_string(free_port());
  const std::string service_frontend = "tcp://127.0.0.1:" + std::to_string(free_port());
  const std::string service_backend = "tcp://127.0.0.1:" + std::to_string(free_port());
  const std::string action_frontend = "tcp://127.0.0.1:" + std::to_string(free_port());
  const std::string action_backend = "tcp://127.0.0.1:" + std::to_string(free_port());
  const std::string api_listen = "127.0.0.1:" + std::to_string(free_port());

  RobotBusBrokerOptions opts{};
  opts.message_xsub_bind = message_xsub.c_str();
  opts.message_xpub_bind = message_xpub.c_str();
  opts.service_frontend_bind = service_frontend.c_str();
  opts.service_backend_bind = service_backend.c_str();
  opts.action_frontend_bind = action_frontend.c_str();
  opts.action_backend_bind = action_backend.c_str();
  opts.api_listen = api_listen.c_str();
  opts.console_listen = nullptr;
  opts.tcp_only = 0;  // keep inproc (+ ipc)
  opts.no_console = 1;
  Broker broker(ctx, opts);
  std::this_thread::sleep_for(std::chrono::milliseconds(150));

  {
    std::atomic<int> hits{0};
    auto sub = Node::inproc_with_context(ctx, "inproc-sub");
    auto sub_h = sub.create_subscription("/inproc/demo", [&](std::string_view, BytesView payload) {
      ROBOT_BUS_CHECK(bytes_to_string(payload) == "hello-inproc");
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
    (void)sub_h;
  }

  {
    auto server = Node::inproc_with_context(ctx, "inproc-action-server");
    auto act = server.create_action_server("/inproc/action", [](BytesView body) {
      const std::string payload = bytes_to_string(body);
      const std::string fb = "step:" + payload;
      const std::string res = "done:" + payload;
      std::vector<std::pair<std::string, std::vector<uint8_t>>> phases;
      phases.emplace_back("FEEDBACK", std::vector<uint8_t>(fb.begin(), fb.end()));
      phases.emplace_back("RESULT", std::vector<uint8_t>(res.begin(), res.end()));
      return phases;
    });
    server.start();
    std::this_thread::sleep_for(std::chrono::milliseconds(100));

    auto client_node = Node::inproc_with_context(ctx, "inproc-action-client");
    auto client = client_node.create_action_client("/inproc/action");
    std::vector<std::string> feedback;
    const std::string goal_body = "move";
    auto handle = client.send_goal(
        goal_body,
        [&](const ActionMessage &message) { feedback.push_back(bytes_to_string(message.body)); },
        nullptr, 3.0);

    ROBOT_BUS_CHECK(handle.action_name() == "/inproc/action");
    ROBOT_BUS_CHECK(!handle.goal_id().empty());
    auto result_message = handle.wait_result(3.0);
    ROBOT_BUS_CHECK(result_message.kind == "RESULT");
    ROBOT_BUS_CHECK(bytes_to_string(result_message.body) == "done:move");
    ROBOT_BUS_CHECK(feedback.size() == 1);
    ROBOT_BUS_CHECK(feedback[0] == "step:move");

    server.shutdown();
    server.wait();
    (void)act;
  }

  broker.stop();
  return 0;
}
