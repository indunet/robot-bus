# *Robot Bus*

[![CI](https://github.com/indunet/robot-bus/actions/workflows/ci.yml/badge.svg)](https://github.com/indunet/robot-bus/actions/workflows/ci.yml)
[![Code Quality](https://img.shields.io/github/actions/workflow/status/indunet/robot-bus/dynamic%2Fgithub-code-scanning%2Fcodeql?label=Code%20Quality)](https://github.com/indunet/robot-bus/security/code-scanning)
[![crates.io](https://img.shields.io/crates/v/robot-bus.svg?color=f74c00)](https://crates.io/crates/robot-bus)
[![PyPI](https://img.shields.io/pypi/v/robot-bus.svg?color=3775a9)](https://pypi.org/project/robot-bus/)
[![npm](https://img.shields.io/npm/v/robot-bus.svg?color=cb3837)](https://www.npmjs.com/package/robot-bus)
[![Maven Central](https://img.shields.io/maven-central/v/org.indunet/robot-bus.svg?label=Maven%20Central&color=007396)](https://central.sonatype.com/artifact/org.indunet/robot-bus)
[![Maven Central (Android)](https://img.shields.io/maven-central/v/org.indunet/robot-bus-android.svg?label=Maven%20Central%20(Android)&color=3ddc84)](https://central.sonatype.com/artifact/org.indunet/robot-bus-android)
[![License](https://img.shields.io/badge/License-Apache_2.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

Lightweight ROS 2–style messaging over ZeroMQ — topics, services & actions, no ROS install. SDKs for Rust, Python, TypeScript, C++, Java, and Android.

轻量级、免环境配置的 ROS 2 风格通信库：基于 ZeroMQ，提供 topic / service / action，以及 `Executor` + `Node` + `spin` 回调模型。多语言 SDK 覆盖 Rust / Python / TypeScript / C++ / Java / Android。

不依赖 ROS 发行版、不需要 `source setup.bash`、不搭 workspace。一个 broker 进程 + 任一语言的 SDK 即可。

**设计原则**：API 会尽量贴近 ROS 2 的用法与命名（如 `Node`、`SingleThreadedExecutor` / `MultiThreadedExecutor`、`add_node`、`create_publisher` / `create_subscription`、`spin`），降低从 ROS 2 迁过来的心智负担；底层用 ZeroMQ 实现，不绑定某一 ROS 发行版。

> **预发布说明**：当前仍处于预发布阶段。接下来 API 可能会有较多变更，运行稳定性也尚不完善，请谨慎用于生产环境。

更多 API 示例见 [`docs/`](docs/)。

| 模块 | 职责 |
|------|------|
| `broker::` | 路由进程（message / service / action） |
| 顶层 API | Publisher / Subscriber / Client / Worker |
| `runtime::Executor` | 底层 poll loop（一般用下面两个包装） |
| `runtime::SingleThreadedExecutor` / `MultiThreadedExecutor` | 显式执行器（多节点 / 并行）；单节点可直接 `Node::spin` |
| `runtime::Node` / `TopicPublisher` / `CallbackGroup` | 节点、publisher、callback group（互斥 / 可重入） |
| `grpc::`（默认 feature） | gRPC / gRPC-Web 网关（随 broker 一起启动） |
| [`proto/`](proto/) | 契约源：ROS 风格 Protobuf → Rust / bindings 生成代码 |
| [`bindings/`](bindings/) | 语言绑定（Python、TypeScript、C++、Java、Android） |
| [`console/`](console/) | Web 监控控制台（产品 UI，嵌入 broker `:15771`；产物在 `assets/console/`） |

## 架构

```
业务代码 (Rust / Python / TypeScript / C++ / Java / Android)
  └── robot-bus SDK
              │
              │ ZMQ (tcp / ipc / inproc) 或 gRPC / gRPC-Web
              ▼
robot_bus_broker 进程
```

## 快速开始

### 1. 启动 broker

Rust：

```bash
cargo run --bin robot_bus_broker
```

Python（`pip install robot-bus` 后自带可执行入口）：

```bash
robot-bus-broker
```

或在代码里进程内启动：

```python
import robot_bus

with robot_bus.RobotBusBroker.start() as broker:
    # ... 业务代码 ...
    pass
# 离开 with 自动 stop

# 或阻塞运行（等同于命令行，Ctrl+C 退出）
# robot_bus.run_broker()
```

### Python

```bash
pip install robot-bus
```

本地开发（需 [maturin](https://www.maturin.rs/)，可选 [just](https://github.com/casey/just)）：

```bash
just python-dev
# 等价：cd bindings/python && maturin develop --features extension-module,grpc
```

（`grpc` 为默认 feature；显式写出可避免 `default-features = false` 的构建漏掉网关。）

```python
import robot_bus
from robot_bus.sensor_msgs.msg.v1 import Imu
from robot_bus.geometry_msgs.msg.v1 import Vector3

def on_imu(topic, imu: Imu):
    print(topic, imu.linear_acceleration)

node = robot_bus.Node("pilot")

imu_pub = node.create_publisher("/robot1/imu", Imu)
node.create_subscription("/robot1/imu", on_imu, msg_type=Imu)
imu_pub.publish(Imu(linear_acceleration=Vector3(x=0.0, y=0.0, z=9.8)))
# node.spin()  # 阻塞；另线程调用 node.shutdown() / shutdown_handle().shutdown()
```

（不传消息类型时仍为 raw bytes。多节点共享或需多线程 handler 时再用 `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node`。）

仅走 gRPC 网关时：`Node.grpc("name")` / `Node.grpc_at("name", "http://…")`（客户端：订阅 / 调 service / action）。详见 [`docs/python-api.md`](docs/python-api.md)。

### TypeScript

```bash
npm install robot-bus
```

本地开发：

```bash
just ts-dev
# 等价：cd bindings/typescript && npm install && npm run build:native && npm run build:ts
```

单一 npm 包：Node.js 走 napi-rs（完整 ZMQ API）；浏览器走 gRPC-Web（仅客户端）。bundler 通过 `exports` 自动选入口。详见 [`docs/typescript-api.md`](docs/typescript-api.md)。

```ts
import { Node } from "robot-bus";
import { Imu } from "robot-bus/sensor_msgs/msg/v1/imu.js";

const node = new Node("pilot");
const pub = node.createPublisher("/robot1/imu", Imu);
node.createSubscription("/robot1/imu", (_t, imu) => console.log(imu), Imu);
```

浏览器 / 纯 gRPC：`Node.grpc("client")`（browser 入口的 `Node` 即为 gRPC-Web facade）。

### Java / Android（Maven Central）

| 产物 | 目录 | 坐标 |
|------|------|------|
| JVM JAR（Java 11+，Maven） | [`bindings/java/`](bindings/java/) | `org.indunet:robot-bus` |
| Android AAR（minSdk 24） | [`bindings/android/`](bindings/android/) | `org.indunet:robot-bus-android` |

包名均为 `org.indunet.robot.bus`。面向 Java 用户；在 GitHub 上写 Release 说明并 Publish 后，CI 会发到 Maven Central（也可手动跑 Actions）。

```bash
just java-dev       # JVM
just android-dev    # AAR（需 Android SDK + NDK 26 + cargo-ndk）
```

```java
// Android
RobotBusAndroid.init(this);
import org.indunet.robot.bus.Node;
import org.indunet.robot.bus.sensor_msgs.msg.v1.Imu;

TypedTopicPublisher<Imu> pub = node.createPublisher("/imu", Imu.class);
```

详见 [`docs/java-api.md`](docs/java-api.md)、[`bindings/java/README.md`](bindings/java/README.md) / [`bindings/android/README.md`](bindings/android/README.md)。

### 2b. C++（DEB / MSI）

C++ 无中央库：从 [GitHub Releases](https://github.com/indunet/robot-bus/releases) 下载 `robot-bus-cpp_*.deb` / `robot-bus-cpp_*.msi` / `robot-bus-cpp_*_darwin-arm64.pkg`（你写 Release 说明并 Publish 后，CI 只挂附件）。详见 [`docs/cpp-api.md`](docs/cpp-api.md)。

```cpp
#include <robot_bus/Node.hpp>
#include <robot_bus/sensor_msgs/msg/v1/imu.pb.h>

robot_bus::Broker broker;
robot_bus::Node node("pilot");
auto pub = node.create_publisher("/imu");
```

### 2. Rust（Node + spin）

在 `Cargo.toml` 中添加依赖：

```toml
robot-bus = { path = "../robot-bus" }
# 或 crates.io：robot-bus = "0.0.6"
```

语义接近 ROS 2：`Node::new` → typed `create_publisher` / `create_subscription` → `node.spin()`（自动挂 `SingleThreadedExecutor`）。

仅走 gRPC 网关（不启 ZMQ）时用 `Node::grpc` / `Node::grpc_at`：可订阅、调 service / action，不能 publish 或当 server；详见 [`docs/rust-api.md`](docs/rust-api.md#grpc-模式-node客户端)。

```rust
use std::sync::Arc;
use std::time::Duration;
use robot_bus::geometry_msgs::msg::v1::Vector3;
use robot_bus::sensor_msgs::msg::v1::Imu;
use robot_bus::Node;

let mut node = Node::new("pilot");

let imu_pub = node.create_publisher::<Imu>("/robot1/imu")?;
node.create_subscription::<Imu, _>(
    "/robot1/imu",
    |topic, imu| {
        println!("{topic}: {:?}", imu.linear_acceleration);
    },
    None,
)?;

let imu = Imu {
    linear_acceleration: Some(Vector3 { x: 0.0, y: 0.0, z: 9.8 }),
    ..Default::default()
};
imu_pub.publish(&imu)?;

node.create_timer(
    Duration::from_millis(100),
    Arc::new(|| {
        // 控制周期 / 心跳
    }),
    None,
)?;

let handle = node.shutdown_handle()?;
std::thread::spawn(move || { /* ... */ handle.shutdown(); });
node.spin()?;
```

- 单节点默认：直接 `node.spin()`（内部 `SingleThreadedExecutor`）
- `SingleThreadedExecutor` / `MultiThreadedExecutor` + `add_node`：多节点共享或并行 handler
- Callback group：`MutuallyExclusive` / `Reentrant`（`create_callback_group`；默认互斥组）
- Service / action：typed `create_service` / `create_client`、`create_action_server` / `create_action_client`（与 topic 一样挂在 Node；另有 `*_raw`）
- Timer：`create_timer`（同样挂在 Node，由 `spin` 驱动）
- Raw bytes：`create_publisher_raw` / `create_subscription_raw`
- 底层 escape hatch：`Executor`（高级用法）
发送 / 接收水位（ZMQ HWM，不是完整 QoS）可在创建时或运行中设置：

```rust
use robot_bus::{Publisher, HighWaterMark};

let pub_ = Publisher::with_hwm(None, HighWaterMark::new(10, 10))?;
pub_.set_high_water_mark(HighWaterMark { snd: 10, rcv: 10 })?;
```

默认：message `STREAM(2/2)`、service `RPC(4/4)`、action `ACTION(8/8)`。Broker 侧用 `--snd-hwm` / `--rcv-hwm`。

## 二进制

| 二进制 | 说明 |
|------|------|
| `robot_bus_broker` | 一次启动三条总线 + gRPC / gRPC-Web 网关 |

## Web 控制台（`console/`）

可选的监控前端：查看 broker 状态、topic 流量与事件日志。默认随 broker 一起启动，监听 `0.0.0.0:15771`（嵌入静态资源，无需再跑 Next.js）。

```bash
cargo run --bin robot_bus_broker
# 浏览器打开 http://localhost:15771
# 关闭控制台：cargo run --bin robot_bus_broker -- --no-console
```

开发时单独跑前端（热更新）：

```bash
cd console
pnpm install   # 或 npm install
pnpm dev       # http://localhost:3000
```

更新嵌入到 broker 的静态资源：

```bash
just console
# 等价：cd console && pnpm build && cd .. && ./scripts/sync_console_assets.sh
# 然后重新 cargo build
```

已对接 broker 同端口监控 API：`GET /api/v1/status`、`GET /api/v1/topics`、`SSE /api/v1/events`。Service / Action 统计尚未接入。不随 crates.io / PyPI 的「源码树」单独发布前端工程，但构建产物会编进带 `console` feature（默认开启）的二进制。

## gRPC / gRPC-Web 网关

随 `robot_bus_broker` / `RobotBusBroker::start` 一起启动；标准 gRPC 与 gRPC-Web **同端口**（默认 `0.0.0.0:15770`）。

也可用 `Node::grpc` / `Node::grpc_at` 以 Node API 接入网关（客户端：订阅 / 调 service / 调 action，见 [`docs/rust-api.md`](docs/rust-api.md#grpc-模式-node客户端)）。

| RPC | 语义 |
|-----|------|
| `MessageGateway.Subscribe` | 按 topic 前缀订阅，服务端流式返回二进制 payload |
| `ServiceGateway.Call` | 一元：`service_name` + request bytes → response bytes |
| `ActionGateway.Run` | 双向流：客户端发 GOAL / CANCEL，服务端推 `ActionEvent`（`kind` 区分 FEEDBACK / RESULT） |

```bash
cargo run --bin robot_bus_broker
# 配置：cargo run --bin robot_bus_broker -- --help
# gRPC: http://0.0.0.0:15770
```

进程内：

```rust
use robot_bus::{GrpcBrokerConfig, RobotBusBroker, RobotBusConfig};

let broker = RobotBusBroker::start(RobotBusConfig {
    grpc: GrpcBrokerConfig {
        listen: "0.0.0.0:15770".parse()?,
        ..Default::default()
    },
    ..RobotBusConfig::default()
})?;
let grpc = format!("http://{}", broker.grpc_listen());
```

Proto（包名 `robot_bus_interface.grpc.v1`，与 ROS `*.msg.v1` / `*.srv.v1` 区分）：

- [`message_gateway.proto`](proto/robot_bus_interface/grpc/v1/message_gateway.proto)
- [`service_gateway.proto`](proto/robot_bus_interface/grpc/v1/service_gateway.proto)
- [`action_gateway.proto`](proto/robot_bus_interface/grpc/v1/action_gateway.proto)

## 测试

```bash
just test-rust
just test-python
just test-typescript
# 等价：
# cargo test
# PYTHONPATH=bindings/python python3 bindings/python/tests/test_msgs_roundtrip.py
# PYTHONPATH=bindings/python python3 bindings/python/tests/test_typed_api.py
# cd bindings/typescript && npm test
```

## Protobuf 消息

[`proto/`](proto/) 按 ROS 包布局：`proto/<pkg>/{msg|srv|grpc}/v1/*.proto`。

各语言 stub **不进 git**；本地改 proto 或跑测试前执行 `just gen-*`（需 protoc **35.1**）。CI / 发版流水线会生成并打进 wheel、crates.io crate、npm 包、DEB/MSI、Maven JAR/AAR——**消费已发布包的用户不需要 protoc**。

| 语言 | 路径 | 说明 |
|------|------|------|
| Rust | `robot_bus::<pkg>::{msg\|srv}::v1` | `just gen-rust` → `src/msgs/generated/`（+ gRPC → `src/grpc/generated/`） |
| Python | `robot_bus.<pkg>.{msg\|srv}.v1` | `just gen-python`；随 wheel 打包 |
| TypeScript | `robot-bus/<pkg>/{msg\|srv}/v1/…` | `just gen-typescript`；随 npm 包打包 |
| Java / Android | `org.indunet.robot.bus.<pkg>.{msg\|srv\|action}.v1` | `just gen-java`；随 JAR / AAR 打包 |
| C++ | `#include <robot_bus/…>` | `just gen-cpp`；随 DEB/MSI 打包 |

- 传输层 body 仍是 opaque bytes（含 gRPC 网关）；Rust Node SDK 在 create 时绑定类型并自动 encode/decode（`create_publisher::<M>` 等），也可用 `*_raw`；Python / TypeScript / **Java** 传入 protobuf 类型即可 typed（薄封装），省略类型则为 raw bytes
- **srv** 是一对 `*Request` / `*Response` message，不是 gRPC
- **grpc**（`robot_bus`）是网关 RPC 契约，随 broker 启动（默认 feature `grpc`）
- 消息在 `robot_bus` 命名空间下，**不占用** ROS 顶层 `sensor_msgs` 包名；编码是 protobuf，与 ROS CDR 不互通
- 一键：`just gen-all`

已覆盖：`builtin_interfaces`、`std_msgs`、`std_srvs`、`geometry_msgs`、`sensor_msgs`、`nav_msgs`、`tf2_msgs`、`trajectory_msgs`、`diagnostic_msgs`、`unique_identifier_msgs`、`shape_msgs`、`visualization_msgs`、`control_msgs`、`nav2_msgs`、`foxglove_msgs`（自 [Foxglove schemas](https://github.com/foxglove/foxglove-sdk) 迁入，包名为 `foxglove_msgs.msg.v1`）。
