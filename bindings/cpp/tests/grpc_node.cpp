// gRPC-mode Node: guards + subscribe / service / action via broker gateway.
#include "harness.hpp"

#include <atomic>
#include <cctype>
#include <functional>
#include <iostream>
#include <string>
#include <utility>
#include <vector>

namespace {

bool contains_ci(std::string_view hay, std::string_view needle) {
  if (needle.empty()) {
    return true;
  }
  for (size_t i = 0; i + needle.size() <= hay.size(); ++i) {
    bool ok = true;
    for (size_t j = 0; j < needle.size(); ++j) {
      if (std::tolower(static_cast<unsigned char>(hay[i + j])) !=
          std::tolower(static_cast<unsigned char>(needle[j]))) {
        ok = false;
        break;
      }
    }
    if (ok) {
      return true;
    }
  }
  return false;
}

void expect_not_supported(const char * /*label*/, const std::function<void()> &fn) {
  try {
    fn();
    ROBOT_BUS_CHECK(false && "expected Error");
  } catch (const robot_bus::Error &e) {
    ROBOT_BUS_CHECK(contains_ci(e.what(), "not supported"));
  }
}

}  // namespace

int main() {
  using namespace robot_bus::test;

  // --- constructors ---
  {
    auto node = robot_bus::Node::grpc("web");
    ROBOT_BUS_CHECK(node.name() == "web");
  }
  {
    auto node = robot_bus::Node::grpc_at("web2", "http://10.0.0.1:15770");
    ROBOT_BUS_CHECK(node.name() == "web2");
  }

  // --- capability guards ---
  {
    auto node = robot_bus::Node::grpc("only-client");
    expect_not_supported("create_publisher", [&] { (void)node.create_publisher("/t"); });
    expect_not_supported("create_service", [&] {
      node.create_service("/svc", [](robot_bus::BytesView) { return std::vector<uint8_t>{}; });
    });
    expect_not_supported("create_action_server", [&] {
      node.create_action_server("/act", [](robot_bus::BytesView) {
        return std::vector<std::pair<std::string, std::vector<uint8_t>>>{};
      });
    });
  }

  // --- subscribe + service via gRPC ---
  {
    auto bus = TestBus::start();
    const std::string grpc_url = "http://" + bus.grpc_listen;

    auto pub_node = bus.make_node("cpp_grpc_pub");
    auto pub = pub_node.create_publisher("cpp.grpc.topic");

    auto server = bus.make_node("svc_server");
    server.create_service("svc.cpp_grpc_echo", [](robot_bus::BytesView body) {
      std::vector<uint8_t> out;
      const char *prefix = "echo:";
      out.insert(out.end(), prefix, prefix + 5);
      out.insert(out.end(), body.data, body.data + body.size);
      return out;
    });
    server.start();
    sleep_ms(200);

    auto client = robot_bus::Node::grpc_at("grpc_client", grpc_url.c_str());
    std::atomic<bool> got{false};
    std::string got_topic;
    std::vector<uint8_t> got_payload;
    client.create_subscription("cpp.grpc.topic", [&](std::string_view topic, robot_bus::BytesView payload) {
      got_topic = std::string(topic);
      got_payload.assign(payload.data, payload.data + payload.size);
      got = true;
    });
    sleep_ms(300);

    const std::string hello = "hello-cpp-grpc";
    pub.publish(hello);
    ROBOT_BUS_CHECK(wait_until([&] {
      client.spin_once(0.05);
      return got.load();
    }));
    ROBOT_BUS_CHECK(got_topic == "cpp.grpc.topic");
    ROBOT_BUS_CHECK(std::string(got_payload.begin(), got_payload.end()) == hello);

    auto svc = client.create_client("svc.cpp_grpc_echo");
    auto reply = svc.call(std::string("ping"), 3.0);
    ROBOT_BUS_CHECK(std::string(reply.begin(), reply.end()) == "echo:ping");

    server.shutdown();
    server.stop();
    server.wait();
    bus.stop();
  }

  // --- action client via gRPC ---
  {
    auto bus = TestBus::start();
    const std::string grpc_url = "http://" + bus.grpc_listen;

    auto server = bus.make_node("act_server");
    server.create_action_server("act.cpp_grpc_demo", [](robot_bus::BytesView body) {
      std::vector<std::pair<std::string, std::vector<uint8_t>>> phases;
      phases.emplace_back("FEEDBACK", std::vector<uint8_t>{'s', 't', 'e', 'p', '-', '1'});
      phases.emplace_back("FEEDBACK", std::vector<uint8_t>{'s', 't', 'e', 'p', '-', '2'});
      std::vector<uint8_t> result;
      const char *prefix = "done:";
      result.insert(result.end(), prefix, prefix + 5);
      result.insert(result.end(), body.data, body.data + body.size);
      phases.emplace_back("RESULT", std::move(result));
      return phases;
    });
    server.start();
    sleep_ms(200);

    auto client = robot_bus::Node::grpc_at("grpc_action", grpc_url.c_str());
    auto action = client.create_action_client("act.cpp_grpc_demo");
    auto messages = action.send_goal(std::string("fly"), nullptr, 5.0);
    ROBOT_BUS_CHECK(messages.size() == 3);
    ROBOT_BUS_CHECK(messages[0].kind == "FEEDBACK");
    ROBOT_BUS_CHECK(std::string(messages[0].body.begin(), messages[0].body.end()) == "step-1");
    ROBOT_BUS_CHECK(messages[2].kind == "RESULT");
    ROBOT_BUS_CHECK(std::string(messages[2].body.begin(), messages[2].body.end()) == "done:fly");

    server.shutdown();
    server.stop();
    server.wait();
    bus.stop();
  }

  std::cout << "ok: grpc_node\n";
  return 0;
}
