#pragma once

#include <robot_bus/node.hpp>

#include <chrono>
#include <cstdlib>
#include <iostream>
#include <memory>
#include <string>
#include <thread>
#include <utility>

namespace robot_bus::test {

inline void fail(const char *file, int line, const char *expr) {
  std::cerr << "CHECK failed at " << file << ":" << line << ": " << expr << "\n";
  std::exit(1);
}

#define ROBOT_BUS_CHECK(expr)                                   \
  do {                                                          \
    if (!(expr)) {                                              \
      ::robot_bus::test::fail(__FILE__, __LINE__, #expr);       \
    }                                                           \
  } while (0)

/// Bind `:0` and let the OS assign ports at `Broker` start.
/// Probing with bind+close (`free_port`) is TOCTOU: the next ZMQ bind can get EADDRINUSE.
inline void set_ephemeral_tcp_binds(RobotBusBrokerOptions &opts) {
  static constexpr const char *kTcp = "tcp://127.0.0.1:0";
  static constexpr const char *kListen = "127.0.0.1:0";
  opts.message_xsub_bind = kTcp;
  opts.message_xpub_bind = kTcp;
  opts.service_frontend_bind = kTcp;
  opts.service_backend_bind = kTcp;
  opts.action_frontend_bind = kTcp;
  opts.action_backend_bind = kTcp;
  opts.api_listen = kListen;
}

inline RobotBusBrokerOptions ephemeral_tcp_opts(int tcp_only, int no_console) {
  RobotBusBrokerOptions opts{};
  set_ephemeral_tcp_binds(opts);
  opts.console_listen = nullptr;
  opts.tcp_only = tcp_only;
  opts.no_console = no_console;
  return opts;
}

/// Owns bind strings + in-process broker on ephemeral TCP ports (tcp_only, no console).
struct TestBus {
  std::string message_xsub;
  std::string message_xpub;
  std::string service_frontend;
  std::string service_backend;
  std::string action_frontend;
  std::string action_backend;
  std::string api_listen;
  std::unique_ptr<Broker> broker;

  static TestBus start() {
    TestBus bus;
    RobotBusBrokerOptions opts = ephemeral_tcp_opts(1, 1);
    bus.broker = std::make_unique<Broker>(opts);
    bus.message_xsub = bus.broker->message_xsub_bind();
    bus.message_xpub = bus.broker->message_xpub_bind();
    bus.service_frontend = bus.broker->service_frontend_bind();
    bus.service_backend = bus.broker->service_backend_bind();
    bus.action_frontend = bus.broker->action_frontend_bind();
    bus.action_backend = bus.broker->action_backend_bind();
    bus.api_listen = bus.broker->api_listen();
    return bus;
  }

  RobotBusNodeOptions node_options() const {
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

  Node make_node(const char *name) const { return Node(name, node_options()); }

  void stop() {
    if (broker) {
      broker->stop();
    }
  }
};

template <typename Pred>
bool wait_until(Pred pred, std::chrono::milliseconds timeout = std::chrono::milliseconds(3000),
                std::chrono::milliseconds step = std::chrono::milliseconds(50)) {
  const auto deadline = std::chrono::steady_clock::now() + timeout;
  while (std::chrono::steady_clock::now() < deadline) {
    if (pred()) {
      return true;
    }
    std::this_thread::sleep_for(step);
  }
  return pred();
}

inline void sleep_ms(int ms) {
  std::this_thread::sleep_for(std::chrono::milliseconds(ms));
}

}  // namespace robot_bus::test
