# C++ binding

C ABI (`native/`) + C++ RAII headers (`include/`) + protobuf msgs under
`generated/robot_bus/` (gitignored; run `just gen-cpp` before build).
Distributed as GitHub Release **DEB / MSI / PKG** assets (see
[docs/cpp-api.md](../../docs/cpp-api.md)).

**C++ standard: C++17** (same minimum as ROS 2 Humble).

## Layout

| Path | Role |
|------|------|
| `native/` | Rust cdylib `robot_bus_c` (installed as `librobot_bus`); optional Cargo feature `ros2` |
| `include/robot_bus.h` | C API (incl. `robot_bus_ros2_*`) |
| `include/robot_bus/` | C++ wrappers (`Node.hpp`, `Ros2Bridge.hpp`, …) |
| `generated/robot_bus/` | `protoc --cpp_out` stubs (gitignored; `just gen-cpp`); built-in action at `robot_bus_interface/action/` |
| `tests/` | C++ tests (pub-sub, service, action, timer, msgs, ros2 stub) |
| `packaging/` | DEB/MSI/PKG packaging (WiX `.wxs`, etc.; used only for C++ Releases) |
| `CMakeLists.txt` | Build/install msgs + link FFI (`-DROBOT_BUS_ROS2=ON` for bridge) |

## Public includes

```cpp
#include <robot_bus/Node.hpp>
#include <robot_bus/Ros2Bridge.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
```

## Build

```bash
just gen-cpp
just cpp-dev          # no ROS
# source /opt/ros/humble/setup.bash   # or jazzy
just cpp-dev-ros2     # --features ros2
```

Release DEBs: `robot-bus-cpp` (no bridge) and `robot-bus-cpp-ros2-{humble,jazzy}` (system ROS, no vendored `rcl`).

## Regenerate msgs

```bash
just gen-cpp
# or: python3 scripts/generate_cpp_msgs.py
```

Requires **protoc 35.1** (same as Python / TypeScript codegen).
