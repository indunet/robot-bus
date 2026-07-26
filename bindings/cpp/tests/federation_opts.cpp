#include "harness.hpp"

#include <string>

int main() {
  using namespace robot_bus;
  using namespace robot_bus::test;

  // Invalid peer XPUB port 0 must fail before broker starts.
  {
    RobotBusBrokerOptions opts{};
    opts.tcp_only = 1;
    opts.no_console = 1;
    const char *bad[] = {"tcp://127.0.0.1:0"};
    opts.message_peers = bad;
    opts.message_peer_count = 1;
    bool threw = false;
    try {
      Broker broker(opts);
    } catch (const Error &e) {
      threw = true;
      ROBOT_BUS_CHECK(std::string(e.what()).find("invalid message peer") != std::string::npos);
    }
    ROBOT_BUS_CHECK(threw);
  }

  // Valid federation peer strings should start successfully (peer need not be up).
  {
    std::string xsub = "tcp://127.0.0.1:" + std::to_string(free_port());
    std::string xpub = "tcp://127.0.0.1:" + std::to_string(free_port());
    std::string svc_fe = "tcp://127.0.0.1:" + std::to_string(free_port());
    std::string svc_be = "tcp://127.0.0.1:" + std::to_string(free_port());
    std::string act_fe = "tcp://127.0.0.1:" + std::to_string(free_port());
    std::string act_be = "tcp://127.0.0.1:" + std::to_string(free_port());
    std::string grpc = "127.0.0.1:" + std::to_string(free_port());
    std::string peer_xpub = "tcp://127.0.0.1:" + std::to_string(free_port());

    RobotBusBrokerOptions opts{};
    opts.message_xsub_bind = xsub.c_str();
    opts.message_xpub_bind = xpub.c_str();
    opts.service_frontend_bind = svc_fe.c_str();
    opts.service_backend_bind = svc_be.c_str();
    opts.action_frontend_bind = act_fe.c_str();
    opts.action_backend_bind = act_be.c_str();
    opts.grpc_listen = grpc.c_str();
    opts.tcp_only = 1;
    opts.no_console = 1;
    opts.broker_id = "broker-a";
    const char *message_peers[] = {peer_xpub.c_str()};
    opts.message_peers = message_peers;
    opts.message_peer_count = 1;

    Broker broker(opts);
    ROBOT_BUS_CHECK(broker.message_xsub_bind().rfind("tcp://", 0) == 0);
    broker.stop();
  }

  return 0;
}
