# C++ binding

C ABI (`native/`) + C++ RAII headers (`include/`) + protobuf msgs under
`generated/robot_bus/` (gitignored; run `just gen-cpp` before build).
Distributed as GitHub Release **DEB / MSI / PKG** assets (see
[docs/cpp-api.md](../../docs/cpp-api.md)).

**C++ standard: C++17** (same minimum as ROS 2 Humble).

## Layout

| Path | Role |
|------|------|
| `native/` | Rust cdylib `robot_bus_c` (installed as `librobot_bus`) |
| `include/robot_bus.h` | C API |
| `include/robot_bus/` | C++ wrappers (`Node.hpp`, …) |
| `generated/robot_bus/` | `protoc --cpp_out` stubs (gitignored; `just gen-cpp`); built-in action at `robot_bus_interface/action/` |
| `tests/` | C++ tests (pub-sub, service, action, timer, msgs) |
| `packaging/` | DEB/MSI/PKG packaging (WiX `.wxs`, etc.; used only for C++ Releases) |
| `CMakeLists.txt` | Build/install msgs + link FFI |

## Public includes

```cpp
#include <robot_bus/Node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.hpp>
```

## Regenerate msgs

```bash
just gen-cpp
# or: python3 scripts/generate_cpp_msgs.py
```

Requires **protoc 35.1** (same as Python / TypeScript codegen).
