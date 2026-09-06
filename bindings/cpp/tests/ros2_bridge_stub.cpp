// Smoke: without ROBOT_BUS_HAS_ROS2, builder.build() throws a clear error.
// Builder `.lazy()` checks run with or without ROS.
// With ROBOT_BUS_HAS_ROS2, also build()+spin_once against an in-process broker.
#include <robot_bus/ros2_bridge.hpp>

#include "harness.hpp"

#include <cstdio>
#include <cstdlib>
#include <memory>
#include <string>

namespace {

robot_bus::TopicQos ros_qos() { return robot_bus::TopicQos::ros_default(); }
robot_bus::TopicQos bus_qos() { return robot_bus::TopicQos::bus(); }

bool qos_eq(const robot_bus::TopicQos &a, const robot_bus::TopicQos &b) {
  return a.depth() == b.depth() && a.is_best_effort() == b.is_best_effort() &&
         a.is_transient_local() == b.is_transient_local();
}

struct AttachOnlyMapper : robot_bus::TopicMapper {};

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

int test_qos_presets() {
  using robot_bus::TopicQos;
  if (!qos_eq(TopicQos::sensor_data(), TopicQos::keep_last(5).best_effort())) {
    std::fprintf(stderr, "sensor_data preset mismatch\n");
    return 1;
  }
  if (!qos_eq(TopicQos::ros_default(), TopicQos::keep_last(10).reliable())) {
    std::fprintf(stderr, "ros_default preset mismatch\n");
    return 1;
  }
  if (!qos_eq(TopicQos::latched(), TopicQos::keep_last(1).reliable().transient_local())) {
    std::fprintf(stderr, "latched preset mismatch\n");
    return 1;
  }
  if (!qos_eq(TopicQos::bus(), TopicQos::keep_last(8).best_effort())) {
    std::fprintf(stderr, "bus preset mismatch\n");
    return 1;
  }
  if (robot_bus::qos_console_label(TopicQos::ros_default()) != "keep_last(10).reliable") {
    std::fprintf(stderr, "qos console label mismatch\n");
    return 1;
  }
  if (robot_bus::qos_console_label(TopicQos::latched()) !=
      "keep_last(1).reliable.transient_local") {
    std::fprintf(stderr, "latched qos console label mismatch\n");
    return 1;
  }
  std::printf("qos presets ok\n");
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

int test_drop_stats_snapshot() {
  robot_bus::DropStatsSnapshot empty;
  if (empty.convert_fail != 0 || empty.decode_fail != 0 || empty.publish_fail != 0) {
    std::fprintf(stderr, "DropStatsSnapshot should start at zero\n");
    return 1;
  }
#ifdef ROBOT_BUS_HAS_ROS2
  robot_bus::DropStats stats;
  stats.convert_fail.fetch_add(1);
  stats.decode_fail.fetch_add(2);
  stats.publish_fail.fetch_add(3);
  auto snap = stats.snapshot();
  if (snap.convert_fail != 1 || snap.decode_fail != 2 || snap.publish_fail != 3) {
    std::fprintf(stderr, "DropStats snapshot mismatch\n");
    return 1;
  }
#endif
#ifdef ROBOT_BUS_HAS_ROS2
  robot_bus::RouteHealth health;
  if (health.take_idle_event(true, false)) {
    std::fprintf(stderr, "idle should wait for grace\n");
    return 1;
  }
  if (!health.take_idle_event(true, true)) {
    std::fprintf(stderr, "idle should fire once after grace\n");
    return 1;
  }
  if (health.take_idle_event(true, true)) {
    std::fprintf(stderr, "idle should latch\n");
    return 1;
  }
  if (!health.should_log_warn() || health.should_log_warn()) {
    std::fprintf(stderr, "warn rate-limit first then silence\n");
    return 1;
  }
#endif
  std::printf("drop_stats snapshot ok\n");
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
  if (int rc = test_qos_presets()) {
    return rc;
  }
  if (int rc = test_qos_builder()) {
    return rc;
  }
  if (int rc = test_service_builder()) {
    return rc;
  }
  if (int rc = test_drop_stats_snapshot()) {
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
      auto drops = bridge.drop_stats();
      if (drops.convert_fail != 0 || drops.decode_fail != 0 || drops.publish_fail != 0) {
        std::fprintf(stderr, "expected zero drop_stats on smoke bridge\n");
        return 1;
      }
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
