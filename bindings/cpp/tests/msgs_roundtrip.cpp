// Pure protobuf serialize/parse (no broker) for generated C++ msgs.
#include "harness.hpp"

#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
#include <robot_bus/std_srvs/srv/v1/set_bool.pb.h>

#include <iostream>
#include <string>

int main() {
  using namespace robot_bus::test;

  {
    sensor_msgs::msg::v1::Imu imu;
    imu.mutable_angular_velocity()->set_x(1.0);
    imu.mutable_angular_velocity()->set_y(2.0);
    imu.mutable_angular_velocity()->set_z(3.0);
    std::string bytes;
    ROBOT_BUS_CHECK(imu.SerializeToString(&bytes));
    sensor_msgs::msg::v1::Imu decoded;
    ROBOT_BUS_CHECK(decoded.ParseFromString(bytes));
    ROBOT_BUS_CHECK(decoded.angular_velocity().z() == 3.0);
  }

  {
    std_srvs::srv::v1::SetBoolRequest req;
    req.set_data(true);
    std::string bytes;
    ROBOT_BUS_CHECK(req.SerializeToString(&bytes));
    std_srvs::srv::v1::SetBoolRequest decoded;
    ROBOT_BUS_CHECK(decoded.ParseFromString(bytes));
    ROBOT_BUS_CHECK(decoded.data());
  }

  {
    example_interfaces::action::v1::FibonacciGoal goal;
    goal.set_order(7);
    std::string bytes;
    ROBOT_BUS_CHECK(goal.SerializeToString(&bytes));
    example_interfaces::action::v1::FibonacciGoal decoded;
    ROBOT_BUS_CHECK(decoded.ParseFromString(bytes));
    ROBOT_BUS_CHECK(decoded.order() == 7);
  }

  std::cout << "ok: msgs_roundtrip\n";
  return 0;
}
