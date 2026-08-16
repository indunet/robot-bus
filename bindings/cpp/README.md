# C++ binding

C ABI (`native/`) + C++ RAII headers (`include/`) + protobuf msgs under
`generated/robot_bus/` (gitignored; run `just gen-cpp` before build).
Distributed as GitHub Release **DEB / MSI / PKG** assets (see
[docs/en/cpp-api.md](../../docs/en/cpp-api.md)).

**C++ standard: C++17** (same minimum as ROS 2 Humble).

**File naming:** hand-written headers/sources use `snake_case` (e.g. `node.hpp`, `ros2_bridge.hpp`). Generated protobuf paths follow `proto/` (`sensor_msgs/msg/v1/imu.pb.h`).

## Layout

| Path | Role |
|------|------|
| `native/` | Rust cdylib `robot_bus_c` (installed as `librobot_bus`) — Node FFI only |
| `include/robot_bus.h` | C API (`robot_bus_ros2_available` is deprecated / always 0) |
| `include/robot_bus/` | C++ wrappers (`node.hpp`, `ros2_bridge.hpp`, …) |
| `src/ros2_bridge.cpp` | Native **rclcpp** Ros2Bridge (`ROBOT_BUS_ROS2=ON`) |
| `generated/robot_bus/` | `protoc --cpp_out` stubs (gitignored; `just gen-cpp`); built-in action at `robot_bus_interfaces/action/` |
| `tests/` | C++ tests (pub-sub, service, action, timer, msgs, ros2 stub) |
| `packaging/` | DEB/MSI/PKG packaging (WiX `.wxs`, etc.; used only for C++ Releases) |
| `CMakeLists.txt` | Build/install msgs + FFI; optional `robot_bus_ros2_bridge` |

## Public includes

```cpp
#include <robot_bus/node.hpp>
#include <robot_bus/ros2_bridge.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
```

## ROS 2 bridge (native rclcpp)

No YAML and no type-name-string route mounting — only `.mapper(concrete object)`.

```cpp
#include <robot_bus/ros2_bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .mapper(robot_bus::StdMsgsStringMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .build();
bridge.spin();
```

`robot_bus::ros2_available()` is true iff the consumer links `robot_bus_ros2_bridge`
(`ROBOT_BUS_HAS_ROS2`). Without that library, `New(...).build()` throws.

Builtins: `StdMsgsStringMapper`, `SensorMsgsImageMapper`, `TriggerServiceMapper`,
`SetBoolServiceMapper`, `FibonacciActionMapper`.

## Build

```bash
just gen-cpp
just cpp-dev          # no ROS
# source /opt/ros/humble/setup.bash   # or jazzy
just cpp-dev-ros2     # -DROBOT_BUS_ROS2=ON (rclcpp; cargo FFI stays without ros2 feature)
```

Release DEBs: `robot-bus` (no bridge) and `robot-bus-ros2-{humble,jazzy}` (system ROS, no vendored `rcl`).

## Regenerate msgs

```bash
just gen-cpp
# or: python3 scripts/generate_cpp_msgs.py
```

Requires **protoc 35.1** (same as Python / TypeScript codegen).
