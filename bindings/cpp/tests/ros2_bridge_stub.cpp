// Smoke: default (no --features ros2) lib reports unavailable and returns clear errors.
#include <robot_bus/Ros2Bridge.hpp>

#include <cstdio>
#include <cstdlib>

int main() {
  if (robot_bus::ros2_available()) {
    // Built with ros2 — stub expectations do not apply.
    std::printf("ros2_bridge_stub: ros2 available (skipping unavailable checks)\n");
    return 0;
  }

  try {
    auto bridge = robot_bus::Ros2Bridge::from_yaml("missing.yaml");
    (void)bridge;
    std::fprintf(stderr, "expected from_yaml to throw when ros2 unavailable\n");
    return 1;
  } catch (const robot_bus::Error &e) {
    std::printf("from_yaml error (ok): %s\n", e.what());
  }

  try {
    auto b = robot_bus::Ros2Bridge::New("x");
    (void)b;
    std::fprintf(stderr, "expected New to throw when ros2 unavailable\n");
    return 1;
  } catch (const robot_bus::Error &e) {
    std::printf("New error (ok): %s\n", e.what());
  }

  std::printf("ros2_bridge_stub ok\n");
  return 0;
}
