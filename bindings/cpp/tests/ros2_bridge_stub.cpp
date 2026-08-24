// Smoke: without ROBOT_BUS_HAS_ROS2, builder.build() throws a clear error.
// Builder `.lazy()` checks run with or without ROS.
#include <robot_bus/ros2_bridge.hpp>

#include <cstdio>
#include <cstdlib>
#include <memory>
#include <string>

namespace {

struct AttachOnlyMapper : robot_bus::TopicMapper {
  const char *type_name() const override { return "test/msg/Dummy"; }
};

bool throws_containing(const char *what, const std::string &haystack) {
  return haystack.find(what) != std::string::npos;
}

int test_lazy_builder() {
  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .route("/a", "/a")
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
                 .route("/a", "/a")
                 .mapper(robot_bus::StdMsgsStringMapper{})
                 .direction(robot_bus::Direction::BusToRos2)
                 .lazy()
                 .add();
    (void)b;
    std::fprintf(stderr, "expected .lazy() + BusToRos2 to throw\n");
    return 1;
  } catch (const robot_bus::Error &e) {
    if (!throws_containing("lazy", e.what()) || !throws_containing("Ros2ToBus", e.what())) {
      std::fprintf(stderr, "wrong BusToRos2 lazy error: %s\n", e.what());
      return 1;
    }
  }

  try {
    auto b = robot_bus::Ros2Bridge::New("t")
                 .route("/a", "/a")
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

}  // namespace

int main() {
  if (int rc = test_lazy_builder()) {
    return rc;
  }

  if (robot_bus::ros2_available()) {
    std::printf("ros2_bridge_stub: ros2 available (skipping unavailable checks)\n");
    return 0;
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
