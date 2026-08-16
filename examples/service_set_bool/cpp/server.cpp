// Service server for /examples/set_bool.
#include <robot_bus/node.hpp>
#include <robot_bus/typed.hpp>
#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>

#include <iostream>
#include <string>

int main() {
  robot_bus::Node node("examples_set_bool_server");
  auto svc = robot_bus::create_service<std_srvs::srv::v1::SetBoolRequest,
                                       std_srvs::srv::v1::SetBoolResponse>(
      node, "/examples/set_bool",
      [](const std_srvs::srv::v1::SetBoolRequest &req) {
        std_srvs::srv::v1::SetBoolResponse resp;
        resp.set_success(true);
        resp.set_message(std::string("set:") + (req.data() ? "true" : "false"));
        return resp;
      });
  (void)svc;
  std::cout << "serving /examples/set_bool (Ctrl+C to stop)\n";
  node.start();
  node.spin();
  return 0;
}
