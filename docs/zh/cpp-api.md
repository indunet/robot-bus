[English](../en/cpp-api.md) | 中文

# C++ API

C++ SDK通过 **GitHub Release附件** 分发（无中央 C++包仓库）。

## 安装包

| 资产 | ROS2 bridge | 说明 |
|------|--------------|------|
| `robot-bus_<ver>_linux_<arch>.deb` | 否 | 默认 SDK + broker。Windows / macOS另有 MSI / PKG。 |
| `robot-bus-ros2-humble_<ver>_linux_<arch>.deb` | 是（Humble） | Release CI **仅 Linux**。需系统已装 **ROS2 Humble**。无 Windows/macOS ros2安装包。 |
| `robot-bus-ros2-jazzy_<ver>_linux_<arch>.deb` | 是（Jazzy） | Release CI **仅 Linux**。需系统已装 **ROS2 Jazzy**。 |

三个 Debian包**互斥**（共用 `librobot_bus.so`）。只装其中一个。

预编译 SDK 压缩包（`.tar.gz` / `.zip`）是给 CMake 工程用的另一份下载，见[预编译 SDK 压缩包](#预编译-sdk-压缩包cmake-fetchcontent)。

**不 vendor `rcl`。** Ros2包动态链接机器上已有的 `rcl` / typesupport。安装后：

```bash
source /opt/ros/humble/setup.bash   # 或 jazzy
# 然后运行使用 Ros2Bridge的二进制
```

你在 GitHub上撰写 Release说明并 Publish；CI只构建包并上传资产。

## 语言 / 工具链

| 要求 | 值 |
|------|-----|
| **C++标准** | **C++17**（与 [ROS2 Humble](https://docs.ros.org/en/humble/How-To-Guides/Ament-CMake-Documentation.html) 对齐） |
| 编译器 | 支持 C++17的 GCC / Clang / MSVC（Humble基线） |

CMake设置 `CMAKE_CXX_STANDARD 17`。自有应用用更高标准（如 `-DCMAKE_CXX_STANDARD=20`）没问题；已安装头文件本身只需 C++17。

## 安装

```bash
# 核心 SDK（无 ROS bridge）
sudo apt install ./robot-bus_3.3.0_linux_amd64.deb

# 或 ROS2 bridge变体（Humble示例）— 需已安装 Humble
sudo apt install ./robot-bus-ros2-humble_3.3.0_linux_amd64.deb
source /opt/ros/humble/setup.bash

# macOS Apple Silicon（仅核心包）
sudo installer -pkg robot-bus_3.3.0_macos_arm64.pkg -target /
# 安装于 /usr/local（{bin,lib,include}）

# 或从源码（开发）
just gen-cpp
just cpp-dev              # 无 ros2
# 带 bridge（先 source Humble或 Jazzy）：
just cpp-dev-ros2
```

头文件安装于 `robot_bus/`前缀下（无 `generated/`段）：

```cpp
#include <robot_bus/node.hpp>
#include <robot_bus/ros2_bridge.hpp>   // 链接 robot_bus_ros2_bridge（ROBOT_BUS_HAS_ROS2）
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
```

链接 `-lrobot_bus -lrobot_bus_msgs`（或 CMake `find_package(robot_bus)` → `robot_bus::robot_bus` + `robot_bus::msgs`）。

系统安装后应能立即用头文件编译：

| 平台 | include / lib根目录 | 典型 CMake |
|------|----------------------|------------|
| Linux (DEB) | `/usr/include`、`/usr/lib` | `find_package(robot_bus REQUIRED)`或 `pkg-config --cflags --libs robot_bus` |
| macOS (PKG) | `/usr/local/include`、`/usr/local/lib` | 同上；默认搜索路径含 `/usr/local` |
| Windows (MSI) | `C:\Program Files\robot-bus\include`（+ `lib`，`bin`在 PATH） | `-DCMAKE_PREFIX_PATH="C:/Program Files/robot-bus"` |

安装包（`.deb` / `.msi` / `.pkg`）和预编译 SDK 压缩包带上 Protobuf **35.1**（`scripts/protobuf-versions.env` 里的 `CPP_PROTOC`），包括头文件、库和 CMake 配置（Windows 的 DLL 在 `bin/`）。对着安装包或这份压缩包编译时用这一份。从本仓库源码构建时，如果找到的 protobuf 和预生成头文件不一致，会重新生成 C++ stubs（`ROBOT_BUS_GENERATE_MSGS=AUTO`）。编译器至少要 3.15，因为 proto 用了 proto3 `optional`。Ubuntu 22.04 的 3.12 不够；Ubuntu 24.04 的系统 protobuf 可以。typed msgs 链接 `robot_bus_msgs`。

运行时检查：`robot_bus::ros2_available()` / `robot_bus_ros2_available()` — 默认包为 `0`，ros2-* 包为 `1`。

## 预编译 SDK 压缩包（CMake FetchContent）

压缩包是编好的 SDK，不是安装程序。配置你自己的工程时，CMake 会下载并解压到构建目录。不需要运行安装包，也不需要自己编译 robot-bus 或安装 Rust 工具链。`.deb` / `.msi` / `.pkg` 仍然可以装到系统里。

| 资产 | 内容 |
|------|------|
| `robot-bus-sdk_<ver>_linux_amd64.tar.gz` | Linux x86-64。在 Ubuntu 22.04 上构建（glibc 2.35 及以上）。 |
| `robot-bus-sdk_<ver>_linux_arm64.tar.gz` | Linux arm64。glibc 要求相同。 |
| `robot-bus-sdk_<ver>_windows_x64.zip` | Windows x64。Release DLL。 |
| `robot-bus-sdk_<ver>_macos_arm64.tar.gz` | macOS Apple Silicon。 |
| `robot-bus-sdk-ros2-humble_<ver>_linux_<arch>.tar.gz` | 核心 SDK 加上 `librobot_bus_ros2_bridge`，仅 Linux。编译和运行都需要系统里的 ROS 2 Humble。 |
| `robot-bus-sdk-ros2-jazzy_<ver>_linux_<arch>.tar.gz` | 同上，编译和运行需要 Ubuntu 24.04 对应的 ROS 2 Jazzy。 |

`<arch>` 为 `amd64` 或 `arm64`。每个压缩包旁边有一个 `.sha256` 文件（`<哈希>  <文件名>`）。压缩包根目录就是前缀：`bin/`、`include/`、`lib/`、`lib/cmake/robot_bus/`。

### 最小 CMakeLists.txt

下面的哈希是 3.0.0 Linux amd64 核心包的。换版本或换平台时，用对应 `.sha256` 附件的第一段。

```cmake
cmake_minimum_required(VERSION 3.22)
project(my_robot LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)

if(POLICY CMP0135)
  cmake_policy(SET CMP0135 NEW)
endif()
include(FetchContent)

FetchContent_Declare(robot_bus_sdk
  URL "https://github.com/indunet/robot-bus/releases/download/v3.0.0/robot-bus-sdk_3.0.0_linux_amd64.tar.gz"
  URL_HASH "SHA256=e375ec6e52cf69b49ece4db6b51336574d9f7cf5908718d12a9b3e0e04ebeb94")
FetchContent_MakeAvailable(robot_bus_sdk)

find_package(robot_bus CONFIG REQUIRED
  PATHS "${robot_bus_sdk_SOURCE_DIR}"
  NO_DEFAULT_PATH)

add_executable(my_robot main.cpp)
target_link_libraries(my_robot PRIVATE robot_bus::robot_bus robot_bus::msgs)
```

```cpp
#include <robot_bus/node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

#include <iostream>
#include <string>

int main() {
  sensor_msgs::msg::v1::Imu imu;
  imu.mutable_angular_velocity()->set_z(3.0);
  std::string bytes;
  imu.SerializeToString(&bytes);

  auto node = robot_bus::Node::inproc("my-robot");
  std::cout << node.name() << " " << bytes.size() << "\n";
}
```

```bash
sudo apt install build-essential cmake
cmake -S . -B build
cmake --build build
./build/my_robot
```

`cmake -S` 下载并解压，`cmake --build` 编译并链接预编译库。直接运行 `./build/my_robot` 即可，不用再设置 `LD_LIBRARY_PATH`。Windows 和 macOS 写法相同，换成对应平台的文件名（`robot-bus-sdk_<ver>_windows_x64.zip`、`robot-bus-sdk_<ver>_macos_arm64.tar.gz`）。

### 文件放在哪里

默认解压到你的构建目录：

```text
build/_deps/robot_bus_sdk-src/
```

`${robot_bus_sdk_SOURCE_DIR}` 就是这个目录。不会写到 `/usr` 或 `Program Files`。只要留着构建目录，之后的配置会复用已解压的内容。删掉构建目录会重新下载。

多个工程共用一份，或离线使用：

```bash
cmake -S . -B build -DFETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK=/opt/robot-bus-sdk-3.0.0
```

这个变量指向解压后的根目录（里面有 `lib/cmake/robot_bus` 的那一层）。

### 按本机选择压缩包

`FetchRobotBus.cmake` 附在每个 GitHub Release 上，压缩包里也有一份：`lib/cmake/robot_bus/FetchRobotBus.cmake`。把它拷进自己的工程（例如 `cmake/FetchRobotBus.cmake`）：

`SHA256` 必须是**当前这台机器会下载的那一个**压缩包的哈希。下面的值只适用于 Linux amd64 核心包。

```cmake
cmake_minimum_required(VERSION 3.22)
project(my_robot LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)

include(cmake/FetchRobotBus.cmake)
robot_bus_fetch_sdk(VERSION 3.0.0
  SHA256 e375ec6e52cf69b49ece4db6b51336574d9f7cf5908718d12a9b3e0e04ebeb94)

add_executable(my_robot main.cpp)
target_link_libraries(my_robot PRIVATE robot_bus::robot_bus robot_bus::msgs)
robot_bus_copy_runtime_libs(my_robot)
```

辅助模块按编译目标选择 `linux` / `windows` / `macos` 和 `amd64` / `arm64` / `x64`。`robot_bus_copy_runtime_libs` 在 Windows 上把 DLL 拷到可执行文件旁边，在 Linux 和 macOS 上把安装用的 RPATH 设为 `$ORIGIN` / `@loader_path`。

镜像或本地缓存：

```bash
cmake -S . -B build -DROBOT_BUS_SDK_BASE_URL=https://mirror.example/robot-bus
# 或者不改 URL，直接用已经解压的目录：
cmake -S . -B build -DFETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK=/opt/robot-bus-sdk-3.0.0
```

`ROBOT_BUS_SDK_BASE_URL` 也可以放在环境变量里。该地址下的文件名与 GitHub Release 上的相同。

### 运行和发布你的程序

Linux 和 macOS 的构建产物通过 RPATH 找到 SDK 里的库，直接运行 `./build/my_robot` 时不用设置 `LD_LIBRARY_PATH`。安装或拷走的二进制需要把 SDK 的 `lib/` 放在程序旁边（`robot_bus_copy_runtime_libs` 会设置 `$ORIGIN` / `@loader_path`），或者保留指向解压目录的 RPATH。

Windows 加载器在 `.exe` 旁边找 DLL。每次构建后用 `robot_bus_copy_runtime_libs` 把 SDK `bin/` 里的 DLL 拷过去。

### 带 ROS 2

配置前和运行前都要 source 对应发行版。`ROS2` 取 `humble` 或 `jazzy`，并链接 `robot_bus_ros2`。压缩包不含 ROS 本身。无 ROS 的工程不要传 `ROS2`，那会去下载核心包。

Humble 的 Linux amd64 哈希如下。Jazzy 或其他架构改用该附件旁边的 `.sha256`。系统里要能 `find_package` 到桥用到的 ROS 包：`rclcpp`、`rclcpp_action`、`std_msgs`、`sensor_msgs`、`geometry_msgs`、`nav_msgs`、`std_srvs`、`example_interfaces`、`builtin_interfaces`、`action_msgs`、`diagnostic_msgs`、`shape_msgs`、`stereo_msgs`、`tf2_msgs`、`trajectory_msgs`、`unique_identifier_msgs`、`visualization_msgs`。`ros-humble-desktop` 已经包含这些；只有 `ros-base` 时再安装对应的 `ros-humble-*` 包。

```cmake
cmake_minimum_required(VERSION 3.22)
project(my_robot LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)

include(cmake/FetchRobotBus.cmake)
robot_bus_fetch_sdk(VERSION 3.0.0 ROS2 humble
  SHA256 65b5c0eead9331b3615c4145c733bf0fae6051d76f2e1197bed86fc03f04867e)

add_executable(my_robot main.cpp)
target_link_libraries(my_robot PRIVATE
  robot_bus::robot_bus
  robot_bus::msgs
  robot_bus_ros2)
robot_bus_copy_runtime_libs(my_robot)
```

```cpp
#include <robot_bus/node.hpp>
#include <robot_bus/ros2_bridge.hpp>

#include <iostream>

int main() {
  if (!robot_bus::ros2_available()) {
    return 1;
  }
  auto node = robot_bus::Node::inproc("my-robot");
  std::cout << node.name() << "\n";
}
```

```bash
source /opt/ros/humble/setup.bash
cmake -S . -B build
cmake --build build
./build/my_robot
```

Jazzy 把 `ROS2 humble` 换成 `ROS2 jazzy`，source `/opt/ros/jazzy/setup.bash`，哈希换成 `robot-bus-sdk-ros2-jazzy_3.0.0_linux_<arch>.tar.gz.sha256` 里的值。Jazzy 压缩包在 Ubuntu 24.04 上构建。

### 限制

- 需要 CMake **3.22** 或更高，包括 Ubuntu 22.04 自带的 CMake 3.22。
- Linux 核心包和 Humble 包在 Ubuntu 22.04 上构建，需要 **glibc 2.35** 或更高。Jazzy 包在 Ubuntu 24.04 上构建，需要 **glibc 2.39** 或更高。
- Windows 只提供 Release DLL。包配置会把 Debug（以及其他非 Release 配置）映射到这些 Release 库。不要再和自己编译的 Debug Protobuf 混用。
- 压缩包内是 Protobuf **35.1**。请链接这一份（压缩包里的 `protobuf::libprotobuf`）。同一进程里再放另一个版本的 Protobuf 会冲突。
- macOS 压缩包是 Apple Silicon。ROS 2 压缩包只有 Linux 上的 Humble 和 Jazzy。

## Broker

**更鼓励在程序里启动 broker**。CLI 适合演示、多进程联调，或单独拉起常驻 broker。

```cpp
#include <robot_bus/node.hpp>

robot_bus::Broker broker;  // 默认 bind
// broker.message_xsub_bind() / api_listen() …
```

需要独立进程时再用 CLI：

```bash
robot_bus_broker
```

跨 broker（federation）通过 `RobotBusBrokerOptions`（与 CLI同款字符串）：

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

同进程 **inproc** 必须与嵌入式 broker共享 `Context`：

```cpp
robot_bus::Context ctx;
robot_bus::Broker broker(ctx);
auto node = robot_bus::Node::inproc_with_context(ctx, "pilot");
```

tcp / ipc / ws不需要共享 context。

### HTTP发现（填充地址，自行选传输）

对已知 API口请求 `GET /api/v1/discover`，拿到可连接的 ZMQ端点。传输仍由你指定：

```cpp
RobotBusDiscoverOpts d{};
d.api_url = "http://127.0.0.1:15560";
// d.broker_id / d.timeout_secs可选；nullptr / 0 → 默认
auto node = robot_bus::Node::discover("talker", "tcp", &d);
// RobotBusBrokerOptions.no_discovery / domain_id为兼容软标签，非 UDP组播
```

## 本地参数

```cpp
robot_bus::Node node("pilot");
node.declare_parameter("max_speed", 1.5);
node.declare_parameter("frame_id", "base_link");
node.set_parameter("max_speed", 2.0);
auto v = std::get<double>(node.get_parameter("max_speed"));
node.load_parameters_from_yaml_str("ros__parameters:\n  max_speed: 3.0\n");
// node.load_parameters_from_yaml("config/pilot.yaml");
```

标量：`bool` / `int64_t` / `double` / `string`。YAML支持扁平 map或 `ros__parameters` / `"/**"`。

## 消息总线（typed）

Protobuf消息类型**预生成**并随包发布 — 仅消费 SDK的机器**不需要** `protoc`。

推荐用 [`robot_bus/typed.hpp`](../../bindings/cpp/include/robot_bus/typed.hpp) 薄封装（`MessageLite`编解码；解码失败跳过并打 log）：

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
// sub.destroy();  // 或依赖 SubscriptionHandle析构
```

完整可运行程序：[`examples/topic_imu/`](../../examples/topic_imu/)、[`examples/service_set_bool/`](../../examples/service_set_bool/)、[`examples/action_fibonacci/`](../../examples/action_fibonacci/)。

原生订阅用 `qos_depth` 设置待处理队列，满时丢旧留新。发布及原生 RPC 的深度用于套接字 HWM。发布和订阅均可传最后一个参数 `true` 启用 `transient_local`。

```cpp
auto writer = node.create_publisher("/robot1/map_state", 1, true);
auto reader = node.create_subscription("/robot1/map_state",
    [](robot_bus::BytesView state) { (void)state; }, nullptr, 1, true);
```
`create_wall_timer` = `create_timer`别名。`wait_for_message` / service·action client的 `wait_for_*`见 Node API。

Raw bytes（手动 Serialize/Parse）仍可用：

```cpp
auto pub = node.create_publisher("/imu");
auto sub = node.create_subscription("/imu", [](robot_bus::BytesView payload) {
  sensor_msgs::msg::v1::Imu imu;
  imu.ParseFromArray(payload.data, static_cast<int>(payload.size));
});
```

参数：`list_parameters(prefixes, depth)`返回 `{names, prefixes}`；带值列表用 `list_all_parameters()`；另有 `undeclare_parameter`。

### WebSocket RPC模式 Node（客户端）

`Node::ws` / `Node::ws_at`经 broker WebSocket RPC服务端接入，不创建 ZMQ socket。

| 支持 | 不支持 |
|------|--------|
| `create_subscription` | `create_service` |
| `create_publisher` | `create_action_server` |
| `create_client` | |
| `create_action_client` | |
| `create_timer`、`spin` / `spin_once` / `shutdown` | — |

```cpp
auto node = robot_bus::Node::ws("web-client");
// 或 robot_bus::Node::ws_at("web-client", "http://127.0.0.1:15560");
```

本地覆盖见 `bindings/cpp/tests/ws_node.cpp`（`just test-cpp`）。

各语言 import对照：

| 语言 | 路径 |
|------|------|
| Python | `from robot_bus.sensor_msgs.msg.v1 import Imu` |
| TypeScript | `import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js"` |
| Java / Android | `import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;` |
| C++（ROS msgs） | `#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>` |
| C++（内置 action） | `#include <robot_bus/example_interfaces/action/v1/fibonacci.pb.h>` |

## CMake

```cmake
find_package(robot_bus REQUIRED)
target_link_libraries(my_app PRIVATE robot_bus::robot_bus robot_bus::msgs)
```

## 本地开发

```bash
just gen-cpp          # 写入 bindings/cpp/generated（gitignored；protoc 35.1，CPP_PROTOC）

just cpp-dev          # cargo FFI + cmake msgs/tests（无 ros2）
just cpp-dev-ros2     # 原生 rclcpp bridge库（先 source Humble/Jazzy）
just test-cpp         # msgs / timer / pub-sub / service / action / ros2 stub
```

仓库布局：`just gen-cpp`后在 `bindings/cpp/generated/robot_bus/`生成源码（gitignored）；安装/公开 include去掉 `generated/`段。Release包自带生成头文件 — 消费者不需要 `protoc`。

## ROS2 bridge（`Ros2Bridge`）

完整用法（Rust + Python + C++）：[`ros2-bridge.md`](ros2-bridge.md)。

C++使用**原生 rclcpp**（`robot_bus_ros2_bridge`，编译宏 `ROBOT_BUS_HAS_ROS2`）— 不经 Rust FFI。无 YAML；仅用具体 mapper对象挂路由。

**预构建**包支持的发行版：**Humble** 与 **Jazzy**（**仅 Linux DEB** — Windows MSI / macOS PKG为核心 SDK，带 stub `Ros2Bridge` API）。所有包头文件均包含；仅 ros2-* 库实现 bridge。

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

内置话题 mapper：Humble/Jazzy 核心目录（约 125 个类型）。`#include <robot_bus/ros2_bridge.hpp>` 即可使用（例如 `GeometryMsgsPoseStampedMapper`）。服务/动作内置仍为手写：`TriggerServiceMapper`、`SetBoolServiceMapper`、`FibonacciActionMapper`。  
自定义：先写对齐 ROS的 bus `.proto`并 `protoc`，再继承 `TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` CRTP，只实现 convert方法，用 `.mapper(std::make_shared<…>())`挂载（见 [ros2-bridge.md](ros2-bridge.md)）。高级场景才 override裸 `attach`。  
链接 `ROBOT_BUS_HAS_ROS2`时 `ros2_available()`为 true。默认 `robot-bus` stub会抛出明确的 `robot_bus::Error`。
