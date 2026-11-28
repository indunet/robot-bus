// Protobuf round-trip that links only robot_bus_msgs.
// Used to smoke-test stubs generated against a system protobuf.

#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>

#include <cstdlib>
#include <iostream>
#include <string>

namespace {

void check(bool ok, const char *what) {
  if (!ok) {
    std::cerr << "msgs_proto_only failed: " << what << "\n";
    std::exit(1);
  }
}

}  // namespace

int main() {
  sensor_msgs::msg::v1::Imu imu;
  imu.mutable_linear_acceleration()->set_z(9.8);
  std::string bytes;
  check(imu.SerializeToString(&bytes), "serialize imu");
  sensor_msgs::msg::v1::Imu decoded;
  check(decoded.ParseFromString(bytes), "parse imu");
  check(decoded.linear_acceleration().z() == 9.8, "imu z");

  std_srvs::srv::v1::SetBoolRequest request;
  request.set_data(true);
  check(request.SerializeToString(&bytes), "serialize setbool");
  std_srvs::srv::v1::SetBoolRequest parsed_request;
  check(parsed_request.ParseFromString(bytes) && parsed_request.data(), "setbool");

  example_interfaces::action::v1::FibonacciGoal goal;
  goal.set_order(5);
  check(goal.SerializeToString(&bytes), "serialize goal");
  example_interfaces::action::v1::FibonacciGoal parsed_goal;
  check(parsed_goal.ParseFromString(bytes) && parsed_goal.order() == 5, "goal");

  std::cout << "msgs proto only ok\n";
  return 0;
}
