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
    RobotBusBrokerOptions opts = ephemeral_tcp_opts(1, 1);
    opts.broker_id = "broker-a";
    const char *message_peers[] = {"tcp://127.0.0.1:16561"};
    opts.message_peers = message_peers;
    opts.message_peer_count = 1;

    Broker broker(opts);
    ROBOT_BUS_CHECK(broker.message_xsub_bind().rfind("tcp://", 0) == 0);
    broker.stop();
  }

  return 0;
}
