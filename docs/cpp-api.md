# C++ API

C++ SDK is distributed via **GitHub Release attachments** (no central C++ registry):

- Linux: `robot-bus-cpp_<version>_amd64.deb`, `robot-bus-cpp_<version>_arm64.deb`
- Windows: `robot-bus-cpp_<version>_x64.msi`
- macOS (Apple Silicon): `robot-bus-cpp_<version>_darwin-arm64.pkg`

You write the Release notes on GitHub and Publish; CI only builds packages and uploads assets.

## Language / toolchain

| Requirement | Value |
|-------------|--------|
| **C++ standard** | **C++17** (aligned with [ROS 2 Humble](https://docs.ros.org/en/humble/How-To-Guides/Ament-CMake-Documentation.html)) |
| Compilers | GCC / Clang / MSVC that support C++17 (Humble’s baseline) |

CMake sets `CMAKE_CXX_STANDARD 17`. Building with a newer standard (e.g. `-DCMAKE_CXX_STANDARD=20`) is fine for your own app; the installed headers themselves only need C++17.

## Install

```bash
# Debian / Ubuntu (amd64 example)
sudo apt install ./robot-bus-cpp_0.0.7_amd64.deb
# Depends: libzmq5, libprotobuf*

# macOS Apple Silicon
sudo installer -pkg robot-bus-cpp_0.0.7_darwin-arm64.pkg -target /
# Installs under /usr/local ({bin,lib,include})

# Or from source (dev)
just gen-cpp
just cpp-dev
```

Headers install under the `robot_bus/` prefix (no `generated/` segment):

```cpp
#include <robot_bus/Node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.hpp>
```

Link with `-lrobot_bus -lrobot_bus_msgs` (or CMake `robot_bus::robot_bus` + `robot_bus::msgs`).

## Broker

```bash
robot_bus_broker
# or in-process:
```

```cpp
#include <robot_bus/Node.hpp>

robot_bus::Broker broker;  // default binds
// broker.message_xsub_bind() / grpc_listen() …
```

Same-process **inproc** must share a `Context` with the embedded broker:

```cpp
robot_bus::Context ctx;
robot_bus::Broker broker(ctx);
auto node = robot_bus::Node::inproc_with_context(ctx, "pilot");
```

tcp / ipc / gRPC do not require a shared context.

## Local parameters

```cpp
robot_bus::Node node("pilot");
node.declare_parameter("max_speed", 1.5);
node.declare_parameter("frame_id", "base_link");
node.set_parameter("max_speed", 2.0);
auto v = std::get<double>(node.get_parameter("max_speed"));
node.load_parameters_from_yaml_str("ros__parameters:\n  max_speed: 3.0\n");
// node.load_parameters_from_yaml("config/pilot.yaml");
```

Scalars: `bool` / `int64_t` / `double` / `string`. YAML supports flat maps or `ros__parameters` / `"/**"`.

## Message bus (typed)

Protobuf message types are **pre-generated** and shipped in the package — you do **not** need `protoc` on the machine that only consumes the SDK.

```cpp
#include <robot_bus/Node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.hpp>

robot_bus::Broker broker;
robot_bus::Node node("pilot");
auto pub = node.create_publisher("/imu");

node.create_subscription("/imu", [](std::string_view topic, robot_bus::BytesView payload) {
  sensor_msgs::msg::v1::Imu imu;
  imu.ParseFromArray(payload.data, static_cast<int>(payload.size));
  // …
});

node.start();
// allow subscription to propagate, then:
sensor_msgs::msg::v1::Imu imu;
imu.mutable_angular_velocity()->set_z(0.1);
std::string bytes;
imu.SerializeToString(&bytes);
pub.publish(bytes);
```

### gRPC mode Node（客户端）

`Node::grpc` / `Node::grpc_at` 经 broker gRPC 网关接入，不创建 ZMQ socket。

| 支持 | 不支持 |
|------|--------|
| `create_subscription` | `create_publisher` |
| `create_client` | `create_service` |
| `create_action_client` | `create_action_server` |
| `create_timer`、`spin` / `spin_once` / `shutdown` | — |

```cpp
auto node = robot_bus::Node::grpc("web-client");
// 或 robot_bus::Node::grpc_at("web-client", "http://127.0.0.1:15770");
```

本地覆盖见 `bindings/cpp/tests/grpc_node.cpp`（`just test-cpp`）。

Compare imports across languages:

| Language | Path |
|----------|------|
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |
| Java / Android | `import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;` |
| C++ (ROS msgs) | `#include <robot_bus/sensor_msgs/msg/v1/imu.pb.hpp>` |
| C++ (built-in action) | `#include <robot_bus/robot_bus_interface/action/v1/fibonacci.pb.hpp>` |

## CMake

```cmake
find_package(robot_bus REQUIRED)
target_link_libraries(my_app PRIVATE robot_bus::robot_bus robot_bus::msgs)
```

## Local development

```bash
just gen-cpp          # write bindings/cpp/generated (gitignored; protoc 35.1)

just cpp-dev          # cargo FFI + cmake msgs/tests
just test-cpp         # msgs / timer / pub-sub / service / action
```

Repo layout: generated sources (gitignored) live under `bindings/cpp/generated/robot_bus/` after `just gen-cpp`; install/public includes drop the `generated/` segment. Release packages ship the generated headers — consumers do not need `protoc`.

