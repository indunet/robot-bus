#include "harness.hpp"

#include <cmath>
#include <fstream>
#include <string>
#include <variant>

int main() {
  using namespace robot_bus;
  using namespace robot_bus::test;

  Node node("params");
  node.declare_parameter("max_speed", 1.5);
  node.declare_parameter("frame_id", "base_link");
  node.declare_parameter("enabled", true);
  node.declare_parameter("count", static_cast<int64_t>(3));

  ROBOT_BUS_CHECK(std::get<double>(node.get_parameter("max_speed")) == 1.5);
  ROBOT_BUS_CHECK(std::get<std::string>(node.get_parameter("frame_id")) == "base_link");
  ROBOT_BUS_CHECK(std::get<bool>(node.get_parameter("enabled")));
  ROBOT_BUS_CHECK(std::get<int64_t>(node.get_parameter("count")) == 3);
  ROBOT_BUS_CHECK(node.has_parameter("frame_id"));
  ROBOT_BUS_CHECK(!node.has_parameter("missing"));

  node.set_parameter("max_speed", 2.0);
  ROBOT_BUS_CHECK(std::get<double>(node.get_parameter("max_speed")) == 2.0);

  auto listed = node.list_parameters();
  ROBOT_BUS_CHECK(listed.size() == 4);

  node.load_parameters_from_yaml_str(
      "ros__parameters:\n  max_speed: 3.25\n  extra: hello\n");
  ROBOT_BUS_CHECK(std::fabs(std::get<double>(node.get_parameter("max_speed")) - 3.25) < 1e-9);
  ROBOT_BUS_CHECK(std::get<std::string>(node.get_parameter("extra")) == "hello");

  const char *path = "/tmp/robot_bus_cpp_params_test.yaml";
  {
    std::ofstream out(path);
    out << "count: 9\n";
  }
  node.load_parameters_from_yaml(path);
  ROBOT_BUS_CHECK(std::get<int64_t>(node.get_parameter("count")) == 9);

  return 0;
}
