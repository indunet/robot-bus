// Call /examples/set_bool once.
#include <robot_bus/node.hpp>
#include <robot_bus/typed.hpp>
#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>

#include <chrono>
#include <iostream>
#include <thread>

int main() {
  robot_bus::Node node("examples_set_bool_client");
  auto client = robot_bus::create_client<std_srvs::srv::v1::SetBoolRequest,
                                         std_srvs::srv::v1::SetBoolResponse>(
      node, "/examples/set_bool");
  node.start();
  client.wait_for_service(5.0);
  std::this_thread::sleep_for(std::chrono::milliseconds(200));

  std_srvs::srv::v1::SetBoolRequest req;
  req.set_data(true);
  auto resp = client.call(req, 5.0);
  std::cout << "success=" << resp.success() << " message=" << resp.message() << "\n";
  node.shutdown();
  return 0;
}
