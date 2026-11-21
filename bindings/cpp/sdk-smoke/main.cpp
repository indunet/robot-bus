#include <robot_bus/node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
#ifdef ROBOT_BUS_HAS_ROS2
#include <robot_bus/ros2_bridge.hpp>
#endif

#include <iostream>
#include <string>

// Release-workflow check: protobuf from the SDK archive, plus an in-process node.
int main() {
  sensor_msgs::msg::v1::Imu imu;
  imu.mutable_angular_velocity()->set_z(3.0);
  std::string bytes;
  if (!imu.SerializeToString(&bytes) || bytes.empty()) {
    std::cerr << "protobuf serialize failed\n";
    return 1;
  }
  sensor_msgs::msg::v1::Imu decoded;
  if (!decoded.ParseFromString(bytes) || decoded.angular_velocity().z() != 3.0) {
    std::cerr << "protobuf parse failed\n";
    return 1;
  }

  auto node = robot_bus::Node::inproc("sdk-smoke");
  if (node.name() != "sdk-smoke") {
    std::cerr << "node name mismatch\n";
    return 1;
  }
#ifdef ROBOT_BUS_HAS_ROS2
  auto bridge = robot_bus::Ros2Bridge::New("sdk_smoke_bridge")
                    .bus_ipc()
                    .from_ros("/sdk_smoke_chatter", robot_bus::TopicQos::ros_default())
                    .to_bus("/sdk_smoke_chatter", robot_bus::TopicQos::bus())
                    .mapper(robot_bus::StdMsgsStringMapper{})
                    .add()
                    .build();
  bridge.spin_once(0.0);
#endif
  std::cout << "sdk-smoke ok\n";
  return 0;
}
