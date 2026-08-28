[English](../en/cpp-api.md) | 中文

# C++ API

C++ SDK 通过 **GitHub Release 附件** 分发（无中央 C++ 包仓库）。

## 安装包

| 资产 | ROS 2 bridge | 说明 |
|------|--------------|------|
| `robot-bus_<ver>_linux_<arch>.deb` | 否 | 默认 SDK + broker。Windows / macOS 另有 MSI / PKG。 |
| `robot-bus-ros2-humble_<ver>_linux_<arch>.deb` | 是（Humble） | Release CI **仅 Linux**。需系统已装 **ROS 2 Humble**。无 Windows/macOS ros2 安装包。 |
| `robot-bus-ros2-jazzy_<ver>_linux_<arch>.deb` | 是（Jazzy） | Release CI **仅 Linux**。需系统已装 **ROS 2 Jazzy**。 |

三个 Debian 包**互斥**（共用 `librobot_bus.so`）。只装其中一个。

**不 vendor `rcl`。** Ros2 包动态链接机器上已有的 `rcl` / typesupport。安装后：

```bash
source /opt/ros/humble/setup.bash   # 或 jazzy
# 然后运行使用 Ros2Bridge 的二进制
```

你在 GitHub 上撰写 Release 说明并 Publish；CI 只构建包并上传资产。

## 语言 / 工具链

| 要求 | 值 |
|------|-----|
| **C++ 标准** | **C++17**（与 [ROS 2 Humble](https://docs.ros.org/en/humble/How-To-Guides/Ament-CMake-Documentation.html) 对齐） |
| 编译器 | 支持 C++17 的 GCC / Clang / MSVC（Humble 基线） |

CMake 设置 `CMAKE_CXX_STANDARD 17`。自有应用用更高标准（如 `-DCMAKE_CXX_STANDARD=20`）没问题；已安装头文件本身只需 C++17。

## 安装

```bash
# 核心 SDK（无 ROS bridge）
sudo apt install ./robot-bus_2.1.0_linux_amd64.deb

# 或 ROS 2 bridge 变体（Humble 示例）— 需已安装 Humble
sudo apt install ./robot-bus-ros2-humble_2.1.0_linux_amd64.deb
source /opt/ros/humble/setup.bash

# macOS Apple Silicon（仅核心包）
sudo installer -pkg robot-bus_2.1.0_macos_arm64.pkg -target /
# 安装于 /usr/local（{bin,lib,include}）

# 或从源码（开发）
just gen-cpp
just cpp-dev              # 无 ros2
# 带 bridge（先 source Humble 或 Jazzy）：
just cpp-dev-ros2
```

头文件安装于 `robot_bus/` 前缀下（无 `generated/` 段）：

```cpp
#include <robot_bus/node.hpp>
#include <robot_bus/ros2_bridge.hpp>   // 链接 robot_bus_ros2_bridge（ROBOT_BUS_HAS_ROS2）
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>
```

链接 `-lrobot_bus -lrobot_bus_msgs`（或 CMake `find_package(robot_bus)` → `robot_bus::robot_bus` + `robot_bus::msgs`）。

系统安装后应能立即用头文件编译：

| 平台 | include / lib 根目录 | 典型 CMake |
|------|----------------------|------------|
| Linux (DEB) | `/usr/include`、`/usr/lib` | `find_package(robot_bus REQUIRED)` 或 `pkg-config --cflags --libs robot_bus` |
| macOS (PKG) | `/usr/local/include`、`/usr/local/lib` | 同上；默认搜索路径含 `/usr/local` |
| Windows (MSI) | `C:\Program Files\robot-bus\include`（+ `lib`，`bin` 在 PATH） | `-DCMAKE_PREFIX_PATH="C:/Program Files/robot-bus"` |

仍需匹配的 **libprotobuf**（Linux/macOS 包内与 SDK 库捆绑；Windows DLL 在 `bin/`），typed msgs 需链接 `robot_bus_msgs`。

运行时检查：`robot_bus::ros2_available()` / `robot_bus_ros2_available()` — 默认包为 `0`，ros2-* 包为 `1`。

## Broker

```bash
robot_bus_broker
# 或进程内：
```

```cpp
#include <robot_bus/node.hpp>

robot_bus::Broker broker;  // 默认 bind
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

同进程 **inproc** 必须与嵌入式 broker 共享 `Context`：

```cpp
robot_bus::Context ctx;
robot_bus::Broker broker(ctx);
auto node = robot_bus::Node::inproc_with_context(ctx, "pilot");
```

tcp / ipc / ws 不需要共享 context。

### HTTP 发现（填充地址，自行选传输）

对已知 API 口请求 `GET /api/v1/discover`，拿到可连接的 ZMQ 端点。传输仍由你指定：

```cpp
RobotBusDiscoverOpts d{};
d.api_url = "http://127.0.0.1:15570";
// d.broker_id / d.timeout_secs 可选；nullptr / 0 → 默认
auto node = robot_bus::Node::discover("talker", "tcp", &d);
// RobotBusBrokerOptions.no_discovery / domain_id 为兼容软标签，非 UDP 组播
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

标量：`bool` / `int64_t` / `double` / `string`。YAML 支持扁平 map 或 `ros__parameters` / `"/**"`。

## 消息总线（typed）

Protobuf 消息类型**预生成**并随包发布 — 仅消费 SDK 的机器**不需要** `protoc`。

推荐用 [`robot_bus/typed.hpp`](../../bindings/cpp/include/robot_bus/typed.hpp) 薄封装（`MessageLite` 编解码；解码失败跳过并打 log）：

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
// sub.destroy();  // 或依赖 SubscriptionHandle 析构
```

完整可运行程序：[`examples/topic_imu/`](../../examples/topic_imu/)、[`examples/service_set_bool/`](../../examples/service_set_bool/)、[`examples/action_fibonacci/`](../../examples/action_fibonacci/)。

可选 QoS：`create_publisher(node, topic, qos_depth)` / `create_subscription(..., group, qos_depth)`（`depth > 0` → KeepLast）。  
`create_wall_timer` = `create_timer` 别名。`wait_for_message` / service·action client 的 `wait_for_*` 见 Node API。

Raw bytes（手动 Serialize/Parse）仍可用：

```cpp
auto pub = node.create_publisher("/imu");
auto sub = node.create_subscription("/imu", [](std::string_view topic, robot_bus::BytesView payload) {
  sensor_msgs::msg::v1::Imu imu;
  imu.ParseFromArray(payload.data, static_cast<int>(payload.size));
});
```

参数：`list_parameters(prefixes, depth)` 返回 `{names, prefixes}`；带值列表用 `list_all_parameters()`；另有 `undeclare_parameter`。

### WebSocket RPC 模式 Node（客户端）

`Node::ws` / `Node::ws_at` 经 broker WebSocket RPC 网关接入，不创建 ZMQ socket。

| 支持 | 不支持 |
|------|--------|
| `create_subscription` | `create_service` |
| `create_publisher` | `create_action_server` |
| `create_client` | |
| `create_action_client` | |
| `create_timer`、`spin` / `spin_once` / `shutdown` | — |

```cpp
auto node = robot_bus::Node::ws("web-client");
// 或 robot_bus::Node::ws_at("web-client", "http://127.0.0.1:15570");
```

本地覆盖见 `bindings/cpp/tests/ws_node.cpp`（`just test-cpp`）。

各语言 import 对照：

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
just gen-cpp          # 写入 bindings/cpp/generated（gitignored；protoc 35.1）

just cpp-dev          # cargo FFI + cmake msgs/tests（无 ros2）
just cpp-dev-ros2     # 原生 rclcpp bridge 库（先 source Humble/Jazzy）
just test-cpp         # msgs / timer / pub-sub / service / action / ros2 stub
just check-ros2-shim  # 无完整 ROS 安装时 typecheck ros2
```

仓库布局：`just gen-cpp` 后在 `bindings/cpp/generated/robot_bus/` 生成源码（gitignored）；安装/公开 include 去掉 `generated/` 段。Release 包自带生成头文件 — 消费者不需要 `protoc`。

## ROS 2 bridge（`Ros2Bridge`）

完整用法（Rust + Python + C++）：[`ros2-bridge.md`](ros2-bridge.md)。

C++ 使用**原生 rclcpp**（`robot_bus_ros2_bridge`，编译宏 `ROBOT_BUS_HAS_ROS2`）— 不经 Rust FFI。无 YAML；仅用具体 mapper 对象挂路由。

**预构建**包支持的发行版：**Humble** 与 **Jazzy**（**仅 Linux DEB** — Windows MSI / macOS PKG 为核心 SDK，带 stub `Ros2Bridge` API）。所有包头文件均包含；仅 ros2-* 库实现 bridge。

```cpp
#include <robot_bus/ros2_bridge.hpp>

auto bridge = robot_bus::Ros2Bridge::New("ros_bridge")
    .bus_tcp("localhost")
    .from_ros("/chatter", robot_bus::TopicQos::keep_last(10).reliable())
    .to_bus("/chatter", robot_bus::TopicQos::keep_last(8).best_effort())
    .mapper(robot_bus::StdMsgsStringMapper{})
    .add()
    .service()
    .from_ros("/reset", robot_bus::TopicQos::keep_last(10).reliable())
    .to_bus("/reset")
    .mapper(robot_bus::TriggerServiceMapper{})
    .add()
    .build();
bridge.spin_once(0.01);
```

一期内置：`StdMsgsStringMapper`、`SensorMsgsImageMapper`、`TriggerServiceMapper`、`SetBoolServiceMapper`、`FibonacciActionMapper`。  
自定义：先写对齐 ROS 的 bus `.proto` 并 `protoc`，再继承 `TypedTopicMapper` / `TypedServiceMapper` / `TypedActionMapper` CRTP，只实现 convert 方法，用 `.mapper(std::make_shared<…>())` 挂载（见 [ros2-bridge.md](ros2-bridge.md)）。高级场景才 override 裸 `attach`。  
链接 `ROBOT_BUS_HAS_ROS2` 时 `ros2_available()` 为 true。默认 `robot-bus` stub 会抛出明确的 `robot_bus::Error`。
