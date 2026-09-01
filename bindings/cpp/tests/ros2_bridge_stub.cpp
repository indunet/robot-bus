// Smoke: without ROBOT_BUS_HAS_ROS2, builder.build() throws a clear error.
// Builder `.lazy()` checks run with or without ROS.
// With ROBOT_BUS_HAS_ROS2, also build()+spin_once against an in-process broker.
#include <robot_bus/ros2_bridge.hpp>
#include <robot_bus/ros2_bridge/mappers/geometry_msgs/pose_stamped.hpp>

#include "harness.hpp"

#include <cstdio>
#include <cstdlib>
#include <memory>
#include <string>

namespace {

robot_bus::TopicQos ros_qos() { return robot_bus::TopicQos::keep_last(10).reliable(); }
robot_bus::TopicQos bus_qos() { return robot_bus::TopicQos::keep_last(8).best_effort(); }

struct AttachOnlyMapper : robot_bus::TopicMapper {
  const char *type_name() const override { return "test/msg/Dummy"; }
};

bool throws_containing(const char *what, const std::string &haystack) {
  return haystack.find(what) != std::string::npos;
}

int test_generated_topic_mapper() {
  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .from_ros("/pose", ros_qos())
                 .to_bus("/pose", bus_qos())
                 .mapper(robot_bus::GeometryMsgsPoseStampedMapper{})
                 .add();
    (void)b;
  } catch (const robot_bus::Error &e) {
    std::fprintf(stderr, "generated mapper path threw: %s\n", e.what());
    return 1;
  }
  std::printf("generated topic mapper add() ok\n");
  return 0;
}

int test_lazy_builder() {
  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .from_ros("/a", ros_qos())
                 .to_bus("/a", bus_qos())
                 .mapper(robot_bus::StdMsgsStringMapper{})
                 .lazy()
                 .add();
    (void)b;
  } catch (const robot_bus::Error &e) {
    std::fprintf(stderr, "eager-ok path threw: %s\n", e.what());
    return 1;
  }

  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .from_ros("/a", ros_qos())
                 .to_bus("/a", bus_qos())
                 .mapper(std::make_shared<AttachOnlyMapper>())
                 .lazy()
                 .add();
    (void)b;
    std::fprintf(stderr, "expected .lazy() + attach-only mapper to throw\n");
    return 1;
  } catch (const robot_bus::Error &e) {
    if (!throws_containing("lazy", e.what())) {
      std::fprintf(stderr, "wrong custom lazy error: %s\n", e.what());
      return 1;
    }
  }

  std::printf("lazy builder checks ok\n");
  return 0;
}

int test_qos_builder() {
  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .from_ros("/a", robot_bus::TopicQos::keep_last(20).best_effort())
                 .to_bus("/a", robot_bus::TopicQos::keep_last(4).best_effort())
                 .mapper(robot_bus::StdMsgsStringMapper{})
                 .add();
    (void)b;
  } catch (const robot_bus::Error &e) {
    std::fprintf(stderr, "qos path threw: %s\n", e.what());
    return 1;
  }

  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .from_ros("/a", ros_qos())
                 .to_bus("/a", robot_bus::TopicQos::keep_last(8).reliable())
                 .mapper(robot_bus::StdMsgsStringMapper{})
                 .add();
    (void)b;
    std::fprintf(stderr, "expected bus reliable TopicQos to throw\n");
    return 1;
    } catch (const robot_bus::Error &e) {
    if (!throws_containing("best_effort", e.what())) {
      std::fprintf(stderr, "wrong bus reliable error: %s\n", e.what());
      return 1;
    }
  }

  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .from_ros("/tf_static",
                           robot_bus::TopicQos::keep_last(1).reliable().transient_local())
                 .to_bus("/tf_static", robot_bus::TopicQos::keep_last(1).best_effort())
                 .mapper(robot_bus::StdMsgsStringMapper{})
                 .add();
    (void)b;
  } catch (const robot_bus::Error &e) {
    std::fprintf(stderr, "transient_local path threw: %s\n", e.what());
    return 1;
  }

  std::printf("qos builder checks ok\n");
  return 0;
}

int test_service_builder() {
  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .service()
                 .from_ros("/a", ros_qos())
                 .to_bus("/a", bus_qos())
                 .mapper(robot_bus::TriggerServiceMapper{})
                 .timeout(2.0)
                 .add()
                 .action()
                 .from_bus("/b", bus_qos())
                 .to_ros("/b", ros_qos())
                 .mapper(robot_bus::FibonacciActionMapper{})
                 .add();
    (void)b;
  } catch (const robot_bus::Error &e) {
    std::fprintf(stderr, "service/action path threw: %s\n", e.what());
    return 1;
  }
  std::printf("service builder checks ok\n");
  return 0;
}

}  // namespace

int main() {
  if (int rc = test_lazy_builder()) {
    return rc;
  }
  if (int rc = test_generated_topic_mapper()) {
    return rc;
  }
  if (int rc = test_qos_builder()) {
    return rc;
  }
  if (int rc = test_service_builder()) {
    return rc;
  }

  if (robot_bus::ros2_available()) {
#ifdef ROBOT_BUS_HAS_ROS2
    try {
      RobotBusBrokerOptions opts = robot_bus::test::ephemeral_tcp_opts(0, 1);
      robot_bus::Broker broker(opts);
      auto bridge = robot_bus::Ros2Bridge::New("cpp_ros2_smoke")
                        .bus_ipc()
                        .from_ros("/rb_cpp_smoke_chatter", ros_qos())
                        .to_bus("/rb_cpp_smoke_chatter", bus_qos())
                        .mapper(robot_bus::StdMsgsStringMapper{})
                        .add()
                        .build();
      bridge.spin_once(0.05);
      std::printf("cpp live ros2 bridge ok\n");
    } catch (const robot_bus::Error &e) {
      std::fprintf(stderr, "live ros2 bridge failed: %s\n", e.what());
      return 1;
    } catch (const std::exception &e) {
      std::fprintf(stderr, "live ros2 bridge exception: %s\n", e.what());
      return 1;
    }
    return 0;
#else
    std::printf("ros2_bridge_stub: ros2 available (skipping unavailable checks)\n");
    return 0;
#endif
  }

  try {
    auto bridge = robot_bus::Ros2Bridge::New("x").build();
    (void)bridge;
    std::fprintf(stderr, "expected New(...).build() to throw when ros2 unavailable\n");
    return 1;
  } catch (const robot_bus::Error &e) {
    std::printf("build error (ok): %s\n", e.what());
  }

  std::printf("ros2_bridge_stub ok\n");
  return 0;
}
