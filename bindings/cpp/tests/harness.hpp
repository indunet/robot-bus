#pragma once

#include <robot_bus/Node.hpp>

#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <memory>
#include <string>
#include <thread>
#include <utility>

#if defined(_WIN32)
#include <winsock2.h>
#include <ws2tcpip.h>
#else
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <unistd.h>
#endif

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

inline uint16_t free_port() {
#if defined(_WIN32)
  WSADATA wsa;
  WSAStartup(MAKEWORD(2, 2), &wsa);
#endif
  int fd = static_cast<int>(::socket(AF_INET, SOCK_STREAM, 0));
  ROBOT_BUS_CHECK(fd >= 0);
  sockaddr_in addr{};
  addr.sin_family = AF_INET;
  addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
  addr.sin_port = 0;
  ROBOT_BUS_CHECK(::bind(fd, reinterpret_cast<sockaddr *>(&addr), sizeof(addr)) == 0);
  socklen_t len = sizeof(addr);
  ROBOT_BUS_CHECK(::getsockname(fd, reinterpret_cast<sockaddr *>(&addr), &len) == 0);
  uint16_t port = ntohs(addr.sin_port);
#if defined(_WIN32)
  closesocket(fd);
#else
  ::close(fd);
#endif
  return port;
}

/// Owns bind strings + in-process broker on ephemeral TCP ports (tcp_only, no console).
struct TestBus {
  std::string message_xsub;
  std::string message_xpub;
  std::string service_frontend;
  std::string service_backend;
  std::string action_frontend;
  std::string action_backend;
  std::string grpc_listen;
  RobotBusBrokerOptions broker_opts{};
  std::unique_ptr<Broker> broker;

  static TestBus start() {
    TestBus bus;
    bus.message_xsub = "tcp://127.0.0.1:" + std::to_string(free_port());
    bus.message_xpub = "tcp://127.0.0.1:" + std::to_string(free_port());
    bus.service_frontend = "tcp://127.0.0.1:" + std::to_string(free_port());
    bus.service_backend = "tcp://127.0.0.1:" + std::to_string(free_port());
    bus.action_frontend = "tcp://127.0.0.1:" + std::to_string(free_port());
    bus.action_backend = "tcp://127.0.0.1:" + std::to_string(free_port());
    bus.grpc_listen = "127.0.0.1:" + std::to_string(free_port());

    bus.broker_opts.message_xsub_bind = bus.message_xsub.c_str();
    bus.broker_opts.message_xpub_bind = bus.message_xpub.c_str();
    bus.broker_opts.service_frontend_bind = bus.service_frontend.c_str();
    bus.broker_opts.service_backend_bind = bus.service_backend.c_str();
    bus.broker_opts.action_frontend_bind = bus.action_frontend.c_str();
    bus.broker_opts.action_backend_bind = bus.action_backend.c_str();
    bus.broker_opts.grpc_listen = bus.grpc_listen.c_str();
    bus.broker_opts.console_listen = nullptr;
    bus.broker_opts.tcp_only = 1;
    bus.broker_opts.no_console = 1;

    bus.broker = std::make_unique<Broker>(bus.broker_opts);
    return bus;
  }

  RobotBusNodeOptions node_options() const {
    RobotBusNodeOptions opts{};
    opts.host = nullptr;
    opts.transport = nullptr;
    opts.grpc_url = nullptr;
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
