// std_srvs/SetBool service client round-trip.
#include "harness.hpp"

#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>

#include <iostream>
#include <string>
#include <vector>

int main() {
  using namespace robot_bus::test;

  auto bus = TestBus::start();
  auto server = bus.make_node("svc_server");
  auto client_node = bus.make_node("svc_client");

  auto svc = server.create_service("/set_bool", [](robot_bus::BytesView body) {
    std_srvs::srv::v1::SetBoolRequest req;
    ROBOT_BUS_CHECK(req.ParseFromArray(body.data, static_cast<int>(body.size)));
    std_srvs::srv::v1::SetBoolResponse resp;
    resp.set_success(true);
    resp.set_message(std::string("set:") + (req.data() ? "true" : "false"));
    std::string out;
    ROBOT_BUS_CHECK(resp.SerializeToString(&out));
    return std::vector<uint8_t>(out.begin(), out.end());
  });

  server.start();
  sleep_ms(150);

  auto client = client_node.create_client("/set_bool");
  std_srvs::srv::v1::SetBoolRequest req;
  req.set_data(true);
  std::string req_bytes;
  ROBOT_BUS_CHECK(req.SerializeToString(&req_bytes));

  auto reply = client.call(req_bytes, 5.0);
  std_srvs::srv::v1::SetBoolResponse resp;
  ROBOT_BUS_CHECK(resp.ParseFromArray(reply.data(), static_cast<int>(reply.size())));
  ROBOT_BUS_CHECK(resp.success());
  ROBOT_BUS_CHECK(resp.message() == "set:true");

  server.shutdown();
  server.wait();
  bus.stop();
  (void)svc;
  std::cout << "ok: service_set_bool\n";
  return 0;
}
