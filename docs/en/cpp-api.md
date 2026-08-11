English | [中文](../zh/cpp-api.md)

# C++ API

C++ SDK is distributed via **GitHub Release attachments** (no central C++ registry).

## Packages

| Asset | ROS 2 bridge | Notes |
|-------|--------------|--------|
| `robot-bus_<ver>_linux_<arch>.deb` | No | Default SDK + broker. Also MSI / PKG on Windows / macOS. |
| `robot-bus-ros2-humble_<ver>_linux_<arch>.deb` | Yes (Humble) | **Linux only** in release CI. Requires system **ROS 2 Humble**. No Windows/macOS ros2 installer. |
| `robot-bus-ros2-jazzy_<ver>_linux_<arch>.deb` | Yes (Jazzy) | **Linux only** in release CI. Requires system **ROS 2 Jazzy**. |

The three Debian packages **conflict** with each other (same `librobot_bus.so`). Install exactly one.

**rcl is not vendored.** Ros2 packages dynamically link the `rcl` / typesupport already on the machine. After install:

```bash
source /opt/ros/humble/setup.bash   # or jazzy
# then run your binary that uses Ros2Bridge
```

You write the Release notes on GitHub and Publish; CI only builds packages and uploads assets.

## Language / toolchain

| Requirement | Value |
|-------------|--------|
| **C++ standard** | **C++17** (aligned with [ROS 2 Humble](https://docs.ros.org/en/humble/How-To-Guides/Ament-CMake-Documentation.html)) |
| Compilers | GCC / Clang / MSVC that support C++17 (Humble’s baseline) |

CMake sets `CMAKE_CXX_STANDARD 17`. Building with a newer standard (e.g. `-DCMAKE_CXX_STANDARD=20`) is fine for your own app; the installed headers themselves only need C++17.

## Install

```bash
# Core SDK (no ROS bridge)
sudo apt install ./robot-bus_0.1.9_linux_amd64.deb

# Or ROS 2 bridge variant (Humble example) — needs Humble already installed
sudo apt install ./robot-bus-ros2-humble_0.1.9_linux_amd64.deb
source /opt/ros/humble/setup.bash

# macOS Apple Silicon (core package only)
sudo installer -pkg robot-bus_0.1.9_macos_arm64.pkg -target /
# Installs under /usr/local ({bin,lib,include})

# Or from source (dev)
just gen-cpp
just cpp-dev              # no ros2
# with bridge (source Humble or Jazzy first):
just cpp-dev-ros2
```

Headers install under the `robot_bus/` prefix (no `generated/` segment):

```cpp
#include <robot_bus/node.hpp>
#include <robot_bus/ros2_bridge.hpp>   // link robot_bus_ros2_bridge (ROBOT_BUS_HAS_ROS2)
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
```

Link with `-lrobot_bus -lrobot_bus_msgs` (or CMake `find_package(robot_bus)` → `robot_bus::robot_bus` + `robot_bus::msgs`).

After a system install you should be able to compile against the headers immediately:

| Platform | Include / lib roots | Typical CMake |
|----------|---------------------|----------------|
| Linux (DEB) | `/usr/include`, `/usr/lib` | `find_package(robot_bus REQUIRED)` or `pkg-config --cflags --libs robot_bus` |
| macOS (PKG) | `/usr/local/include`, `/usr/local/lib` | same; `/usr/local` is on default search paths |
| Windows (MSI) | `C:\Program Files\robot-bus\include` (+ `lib`, `bin` on PATH) | `-DCMAKE_PREFIX_PATH="C:/Program Files/robot-bus"` |

You still need a matching **libprotobuf** (bundled next to the SDK libs in the package on Linux/macOS; DLLs under `bin/` on Windows) and, for typed msgs, to link `robot_bus_msgs`.

Check at runtime: `robot_bus::ros2_available()` / `robot_bus_ros2_available()` — `0` on the default package, `1` on ros2-* packages.

## Broker

```bash
robot_bus_broker
# or in-process:
```

```cpp
#include <robot_bus/node.hpp>

robot_bus::Broker broker;  // default binds
// broker.message_xsub_bind() / api_listen() …
```

跨 broker（federation）通过 `RobotBusBrokerOptions`（与 CLI 同款字符串）：

```cpp
const char *message_peers[] = {"tcp://10.0.0.2:15561"};
const char *service_peers[] = {"broker-b=tcp://10.0.0.2:15663"};
const char *action_peers[] = {"broker-b=tcp://10.0.0.2:15665"};

RobotBusBrokerOptions opts{};
opts.broker_id = "broker-a";
opts.message_peers = message_peers;
opts.message_peer_count = 1;
opts.service_peers = service_peers;
opts.service_peer_count = 1;
opts.action_peers = action_peers;
opts.action_peer_count = 1;
opts.tcp_only = 1;
opts.no_console = 1;

robot_bus::Broker broker(opts);
```

Same-process **inproc** must share a `Context` with the embedded broker:

```cpp
robot_bus::Context ctx;
robot_bus::Broker broker(ctx);
auto node = robot_bus::Node::inproc_with_context(ctx, "pilot");
```

tcp / ipc / gRPC do not require a shared context.

### HTTP discovery (fill addresses, pick transport yourself)

Request `GET /api/v1/discover` on a known API base URL to obtain connectable ZMQ endpoints. You still choose the transport:

```cpp
RobotBusDiscoverOpts d{};
d.api_url = "http://127.0.0.1:15570";
// Optional: broker_id / timeout_secs; nullptr / 0 → defaults
auto node = robot_bus::Node::discover("talker", "tcp", &d);
// RobotBusBrokerOptions.no_discovery / domain_id are soft labels, not UDP multicast
```

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

Prefer [`robot_bus/typed.hpp`](../../bindings/cpp/include/robot_bus/typed.hpp) (`MessageLite` encode/decode; decode failures are skipped and logged):

```cpp
#include <robot_bus/node.hpp>
#include <robot_bus/typed.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

robot_bus::Broker broker;
robot_bus::Node node("pilot");
auto pub = robot_bus::create_publisher<sensor_msgs::msg::v1::Imu>(node, "/imu");
auto sub = robot_bus::create_subscription<sensor_msgs::msg::v1::Imu>(
    node, "/imu", [](std::string_view, const sensor_msgs::msg::v1::Imu &imu) {
      // …
    });

node.start();
sensor_msgs::msg::v1::Imu imu;
imu.mutable_angular_velocity()->set_z(0.1);
pub.publish(imu);
// sub.destroy();
```

Optional QoS: `qos_depth > 0` → KeepLast. `create_wall_timer` aliases `create_timer`. See also `wait_for_message` / client `wait_for_*`. Parameters: `list_parameters` → `{names, prefixes}`; `list_all_parameters`; `undeclare_parameter`.

Raw bytes still work via `create_publisher` / `create_subscription` with manual Serialize/Parse.

### gRPC mode Node（客户端）

`Node::grpc` / `Node::ws_at` 经 broker gRPC 网关接入，不创建 ZMQ socket。

| 支持 | 不支持 |
|------|--------|
| `create_subscription` | `create_publisher` |
| `create_client` | `create_service` |
| `create_action_client` | `create_action_server` |
| `create_timer`、`spin` / `spin_once` / `shutdown` | — |

```cpp
auto node = robot_bus::Node::ws("web-client");
// 或 robot_bus::Node::ws_at("web-client", "http://127.0.0.1:15570");
```

本地覆盖见 `bindings/cpp/tests/ws_node.cpp`（`just test-cpp`）。

## TF (coordinate frames)

`TfBuffer` / `TfListener` / `TransformBroadcaster` mirror Rust `robot_bus::tf`. Wire format is `tf2_msgs/TFMessage` on `/tf` and `/tf_static`. Lookup returns `geometry_msgs/TransformStamped` protobuf bytes. v1 time semantics: static edges always apply; dynamic = latest only.

```cpp
#include <robot_bus/tf.hpp>
#include <robot_bus/tf2_msgs/msg/v1/tf_message.pb.h>
#include <robot_bus/geometry_msgs/msg/v1/stamped.pb.h>

robot_bus::TfListener listener(node);  // /tf + /tf_static
auto buf = listener.buffer();

auto pub = node.create_publisher("/tf_static");
robot_bus::TransformBroadcaster br(std::move(pub));
tf2_msgs::msg::v1::TFMessage msg;
// … fill transforms …
std::string bytes;
msg.SerializeToString(&bytes);
br.send(bytes);

// after spin delivers messages:
auto stamped_bytes = buf.lookup_transform("base_link", "camera");
geometry_msgs::msg::v1::TransformStamped stamped;
stamped.ParseFromArray(stamped_bytes.data(), static_cast<int>(stamped_bytes.size()));
```

Offline use: `robot_bus::TfBuffer` + `set_transform_msg` without a listener. See `bindings/cpp/tests/tf_lookup.cpp`.

Compare imports across languages:

| Language | Path |
|----------|------|
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |
| Java / Android | `import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;` |
| C++ (ROS msgs) | `#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>` |
| C++ (built-in action) | `#include <robot_bus/robot_bus_interface/action/v1/fibonacci.pb.h>` |

## CMake

```cmake
find_package(robot_bus REQUIRED)
target_link_libraries(my_app PRIVATE robot_bus::robot_bus robot_bus::msgs)
```

## Local development

```bash
just gen-cpp          # write bindings/cpp/generated (gitignored; protoc 35.1)

just cpp-dev          # cargo FFI + cmake msgs/tests (no ros2)
just cpp-dev-ros2     # native rclcpp bridge lib (source Humble/Jazzy first)
just test-cpp         # msgs / timer / pub-sub / service / action / ros2 stub
just check-ros2-shim  # typecheck ros2 without a full ROS install
```

Repo layout: generated sources (gitignored) live under `bindings/cpp/generated/robot_bus/` after `just gen-cpp`; install/public includes drop the `generated/` segment. Release packages ship the generated headers — consumers do not need `protoc`.

## ROS 2 bridge (`Ros2Bridge`)

Full usage (Rust + Python + C++): [`ros2-bridge.md`](ros2-bridge.md).

C++ uses **native rclcpp** (`robot_bus_ros2_bridge`, compile flag `ROBOT_BUS_HAS_ROS2`) — not Rust FFI. No YAML; mount routes with concrete mapper objects only.

Supported distros for **prebuilt** packages: **Humble** and **Jazzy** (**Linux DEBs only** — Windows MSI / macOS PKG ship core SDK with stub `Ros2Bridge` APIs). Headers ship in all packages; only ros2-* libs implement the bridge.

```cpp
#include <robot_bus/ros2_bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .route("/chatter", "/chatter")
    .mapper(robot_bus::StdMsgsStringMapper{})
    .direction(robot_bus::Direction::Ros2ToBus)
    .add()
    .service("/reset", "/reset")
    .mapper(robot_bus::TriggerServiceMapper{})
    .add()
    .build();
bridge.spin_once(0.01);
```

Phase-1 builtins: `StdMsgsStringMapper`, `SensorMsgsImageMapper`, `TriggerServiceMapper`, `SetBoolServiceMapper`, `FibonacciActionMapper`.  
Custom: inherit `TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` CRTP, implement convert methods only, mount with `.mapper(std::make_shared<…>())` (see [ros2-bridge.md](ros2-bridge.md)). Override bare `attach` only for advanced cases.  
`ros2_available()` is true when linked with `ROBOT_BUS_HAS_ROS2`. Default `robot-bus` stubs throw a clear `robot_bus::Error`.

