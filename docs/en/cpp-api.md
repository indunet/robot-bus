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

Prebuilt SDK archives (`.tar.gz` / `.zip`) are a separate download for CMake projects. See [Prebuilt SDK archive](#prebuilt-sdk-archive-cmake-fetchcontent).

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
sudo apt install ./robot-bus_2.4.2_linux_amd64.deb

# Or ROS 2 bridge variant (Humble example) — needs Humble already installed
sudo apt install ./robot-bus-ros2-humble_2.4.2_linux_amd64.deb
source /opt/ros/humble/setup.bash

# macOS Apple Silicon (core package only)
sudo installer -pkg robot-bus_2.4.2_macos_arm64.pkg -target /
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

Installer packages (`.deb` / `.msi` / `.pkg`) ship the matching Protobuf shared libraries beside the SDK (DLLs under `bin/` on Windows). Compiling your own sources against an installer requires a Protobuf **35.1** development package on the machine. The prebuilt SDK archive below includes Protobuf 35.1 headers, libraries, and CMake config. Typed messages link `robot_bus_msgs`.

Check at runtime: `robot_bus::ros2_available()` / `robot_bus_ros2_available()` — `0` on the default package, `1` on ros2-* packages.

## Prebuilt SDK archive (CMake FetchContent)

The archive is a compiled SDK, not an installer. CMake downloads it while configuring your project and unpacks it into the build directory. You do not run an installer, and you do not build robot-bus or install a Rust toolchain. `.deb` / `.msi` / `.pkg` stay available for a system install.

| Asset | Contents |
|-------|----------|
| `robot-bus-sdk_<ver>_linux_amd64.tar.gz` | Linux x86-64. Built on Ubuntu 22.04 (glibc 2.35 or newer). |
| `robot-bus-sdk_<ver>_linux_arm64.tar.gz` | Linux arm64. Same glibc floor. |
| `robot-bus-sdk_<ver>_windows_x64.zip` | Windows x64. Release DLLs. |
| `robot-bus-sdk_<ver>_macos_arm64.tar.gz` | macOS Apple Silicon. |
| `robot-bus-sdk-ros2-humble_<ver>_linux_<arch>.tar.gz` | Core SDK plus `librobot_bus_ros2_bridge`, Linux only. Needs system ROS 2 Humble at build and run time. |
| `robot-bus-sdk-ros2-jazzy_<ver>_linux_<arch>.tar.gz` | Same for Jazzy. |

`<arch>` is `amd64` or `arm64`. Each archive has a sibling `.sha256` file (`<hash>  <filename>`). The archive root is the prefix: `bin/`, `include/`, `lib/`, and `lib/cmake/robot_bus/`.

### Minimal CMakeLists.txt

Replace the version and hash with the release you want. The hash is the first field of the `.sha256` asset.

```cmake
cmake_minimum_required(VERSION 3.24)
project(my_robot LANGUAGES CXX)

include(FetchContent)

FetchContent_Declare(robot_bus_sdk
  URL "https://github.com/indunet/robot-bus/releases/download/v2.4.2/robot-bus-sdk_2.4.2_linux_amd64.tar.gz"
  URL_HASH SHA256=<hash from robot-bus-sdk_2.4.2_linux_amd64.tar.gz.sha256>
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(robot_bus_sdk)

find_package(robot_bus CONFIG REQUIRED
  PATHS "${robot_bus_sdk_SOURCE_DIR}"
  NO_DEFAULT_PATH)

add_executable(my_robot main.cpp)
target_link_libraries(my_robot PRIVATE robot_bus::robot_bus robot_bus::msgs)
```

```bash
cmake -S . -B build
cmake --build build
```

The first command downloads and unpacks the archive. The second compiles your program and links the prebuilt libraries. Windows and macOS use the same CMake with that platform's archive name (`robot-bus-sdk_<ver>_windows_x64.zip`, `robot-bus-sdk_<ver>_macos_arm64.tar.gz`).

### Where the files go

By default CMake unpacks into your build directory:

```text
build/_deps/robot_bus_sdk-src/
```

`${robot_bus_sdk_SOURCE_DIR}` is that directory. Nothing is written under `/usr` or `Program Files`. Later configures reuse the unpacked tree as long as the build directory is kept. Deleting the build directory downloads it again.

To share one copy across projects, or to work offline:

```bash
cmake -S . -B build -DFETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK=/opt/robot-bus-sdk-2.4.2
```

Point that variable at the unpacked archive root (the directory that contains `lib/cmake/robot_bus`).

### Pick the archive for this machine

`FetchRobotBus.cmake` is on each GitHub Release and inside the archive at `lib/cmake/robot_bus/FetchRobotBus.cmake`. Copy it into your project (for example `cmake/FetchRobotBus.cmake`):

```cmake
cmake_minimum_required(VERSION 3.24)
project(my_robot LANGUAGES CXX)

include(cmake/FetchRobotBus.cmake)
robot_bus_fetch_sdk(VERSION 2.4.2 SHA256 <hash>)
# ROS 2 Humble on Linux: robot_bus_fetch_sdk(VERSION 2.4.2 ROS2 humble SHA256 <hash>)

add_executable(my_robot main.cpp)
target_link_libraries(my_robot PRIVATE robot_bus::robot_bus robot_bus::msgs)
robot_bus_copy_runtime_libs(my_robot)
```

The helper selects `linux` / `windows` / `macos` and `amd64` / `arm64` / `x64` from the compiler target. `robot_bus_copy_runtime_libs` copies DLLs next to the executable on Windows, and records `$ORIGIN` / `@loader_path` as the install RPATH on Linux and macOS.

A mirror or a cached directory:

```bash
cmake -S . -B build -DROBOT_BUS_SDK_BASE_URL=https://mirror.example/robot-bus
# or leave the URL alone and reuse an unpacked tree:
cmake -S . -B build -DFETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK=/opt/robot-bus-sdk-2.4.2
```

`ROBOT_BUS_SDK_BASE_URL` may also be set in the environment. The file name under that URL is the same as on the GitHub Release.

### Running and shipping your binary

Linux and macOS binaries pick up the SDK libraries from the build tree through RPATH, so running `./build/my_robot` needs no `LD_LIBRARY_PATH`. An installed or copied binary needs the SDK `lib/` directory beside it (`robot_bus_copy_runtime_libs` sets `$ORIGIN` / `@loader_path`) or an RPATH that still points at the unpacked archive.

On Windows the loader looks for DLLs next to the `.exe`. Call `robot_bus_copy_runtime_libs` so the SDK `bin/` DLLs are copied there after each build.

ROS 2 archives: source the matching distro before configuring and before running.

```bash
source /opt/ros/humble/setup.bash   # or jazzy
cmake -S . -B build
```

Link `robot_bus::ros2_bridge` in addition to `robot_bus::robot_bus` and `robot_bus::msgs`. The archive does not contain ROS itself.

### Limits

- CMake **3.24** or newer (`FetchContent` `DOWNLOAD_EXTRACT_TIMESTAMP`).
- Linux archives are built on Ubuntu 22.04 and need **glibc 2.35** or newer.
- Windows ships Release DLLs. The package maps Debug (and the other non-Release configs) onto those Release binaries. Do not mix them with a Debug Protobuf you built yourself.
- The archive carries Protobuf **35.1**. Link that copy (`protobuf::libprotobuf` from the archive). A second Protobuf of another version in the same process will clash.
- The macOS archive is Apple Silicon. ROS 2 archives are Linux Humble and Jazzy only.

## Broker

**Prefer starting the broker from your program.** The CLI is for demos, multi-process bring-up, or a standalone long-running broker.

```cpp
#include <robot_bus/node.hpp>

robot_bus::Broker broker;  // default binds
// broker.message_xsub_bind() / api_listen() …
```

Use the CLI when you need a standalone process:

```bash
robot_bus_broker
```

跨 broker（federation）通过 `RobotBusBrokerOptions`（与 CLI 同款字符串）：

```cpp
const char *message_peers[] = {"tcp://10.0.0.2:15581"};
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

tcp / ipc / ws do not require a shared context.

### HTTP discovery (fill addresses, pick transport yourself)

Request `GET /api/v1/discover` on a known API base URL to obtain connectable ZMQ endpoints. You still choose the transport:

```cpp
RobotBusDiscoverOpts d{};
d.api_url = "http://127.0.0.1:15560";
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
    node, "/imu", [](const sensor_msgs::msg::v1::Imu &imu) {
      // …
    });

node.start();
sensor_msgs::msg::v1::Imu imu;
imu.mutable_angular_velocity()->set_z(0.1);
pub.publish(imu);
// sub.destroy();
```

Full runnable programs: [`examples/topic_imu/`](../../examples/topic_imu/), [`examples/service_set_bool/`](../../examples/service_set_bool/), [`examples/action_fibonacci/`](../../examples/action_fibonacci/).

Optional QoS: `qos_depth > 0` → KeepLast (topic PUB/SUB HWM; service / action DEALER HWM). `create_publisher(topic, qos_depth)` / `create_subscription(..., group, qos_depth)` / `create_service(..., group, qos_depth)` / `create_client(name, qos_depth)` / `create_action_server(..., group, qos_depth)` / `create_action_client(name, qos_depth)`. `create_wall_timer` aliases `create_timer`. See also `wait_for_message` / client `wait_for_*`. Parameters: `list_parameters` → `{names, prefixes}`; `list_all_parameters`; `undeclare_parameter`.

Raw bytes still work via `create_publisher` / `create_subscription` with manual Serialize/Parse.

### WebSocket RPC mode Node (client)

`Node::ws` / `Node::ws_at` connect through the broker WebSocket RPC server and do not create ZMQ sockets.

| Supported | Not supported |
|-----------|---------------|
| `create_subscription` | `create_service` |
| `create_publisher` | `create_action_server` |
| `create_client` | |
| `create_action_client` | |
| `create_timer`, `spin` / `spin_once` / `shutdown` | — |

```cpp
auto node = robot_bus::Node::ws("web-client");
// 或 robot_bus::Node::ws_at("web-client", "http://127.0.0.1:15560");
```

本地覆盖见 `bindings/cpp/tests/ws_node.cpp`（`just test-cpp`）。

Compare imports across languages:

| Language | Path |
|----------|------|
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |
| Java / Android | `import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;` |
| C++ (ROS msgs) | `#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>` |
| C++ (built-in action) | `#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>` |

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
    .from_ros("/chatter", robot_bus::TopicQos::ros_default())
    .to_bus("/chatter", robot_bus::TopicQos::bus())
    .mapper(robot_bus::StdMsgsStringMapper{})
    .add()
    .service()
    .from_ros("/reset", robot_bus::TopicQos::ros_default())
    .to_bus("/reset", robot_bus::TopicQos::bus())
    .mapper(robot_bus::TriggerServiceMapper{})
    .add()
    .build();
bridge.spin_once(0.01);
```

Built-in topic mappers: Humble/Jazzy core catalog (~125 types). `#include <robot_bus/ros2_bridge.hpp>` provides them (e.g. `GeometryMsgsPoseStampedMapper`). Service/action builtins remain hand-written: `TriggerServiceMapper`, `SetBoolServiceMapper`, `FibonacciActionMapper`.  
Custom: write a bus `.proto` aligned with the ROS type and `protoc` it, then inherit `TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` CRTP, implement convert methods only, mount with `.mapper(std::make_shared<…>())` (see [ros2-bridge.md](ros2-bridge.md)). Override bare `attach` only for advanced cases.  
`ros2_available()` is true when linked with `ROBOT_BUS_HAS_ROS2`. Default `robot-bus` stubs throw a clear `robot_bus::Error`.

