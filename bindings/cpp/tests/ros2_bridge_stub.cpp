// Smoke: without ROBOT_BUS_HAS_ROS2, builder.build() throws a clear error.
#include <robot_bus/ros2_bridge.hpp>

#include <cstdio>
#include <cstdlib>

int main() {
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
